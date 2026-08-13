import asyncio
import json
import logging
import os
import sqlite3
from typing import Dict, Optional, Set
import httpx
import websockets
from fastapi import FastAPI, HTTPException, BackgroundTasks
from pydantic import BaseModel

logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s")
logger = logging.getLogger("LampcordPushRelay")

DB_FILE = os.getenv("DB_FILE", "relay_users.db")
FCM_CREDENTIALS_FILE = os.getenv("FCM_CREDENTIALS_FILE", "firebase-service-account.json")
FIREBASE_PROJECT_ID = os.getenv("FIREBASE_PROJECT_ID", "")

app = FastAPI(title="Lampcord Multi-User Push Relay API", version="1.0.0")

# Database Initialization
def init_db():
    conn = sqlite3.connect(DB_FILE)
    cursor = conn.cursor()
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS user_devices (
            user_id TEXT,
            discord_token TEXT,
            fcm_token TEXT,
            device_id TEXT,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (discord_token, fcm_token)
        )
    """)
    conn.commit()
    conn.close()

init_db()

class RegisterDeviceRequest(BaseModel):
    discord_token: str
    fcm_token: str
    device_id: str
    user_id: Optional[str] = None

class UnregisterDeviceRequest(BaseModel):
    discord_token: str
    fcm_token: str

class UserSessionManager:
    def __init__(self):
        self.active_tasks: Dict[str, asyncio.Task] = {}  # token -> Task

    async def start_user_session(self, token: str):
        if token in self.active_tasks and not self.active_tasks[token].done():
            return
        task = asyncio.create_task(self._discord_gateway_loop(token))
        self.active_tasks[token] = task

    async def stop_user_session(self, token: str):
        if token in self.active_tasks:
            self.active_tasks[token].cancel()
            del self.active_tasks[token]

    async def _discord_gateway_loop(self, token: str):
        gateway_url = "wss://gateway.discord.gg/?v=9&encoding=json"
        retry_delay = 5

        while True:
            try:
                logger.info(f"Connecting to Discord Gateway for token starting with {token[:8]}...")
                async with websockets.connect(gateway_url) as ws:
                    retry_delay = 5
                    
                    # Read HELLO
                    hello_raw = await ws.recv()
                    hello = json.loads(hello_raw)
                    heartbeat_interval = hello["d"]["heartbeat_interval"] / 1000.0

                    # Identify
                    identify_payload = {
                        "op": 2,
                        "d": {
                            "token": token,
                            "capabilities": 30717,
                            "properties": {
                                "os": "Android",
                                "browser": "Discord-Android",
                                "device": "LampcordRelay"
                            },
                            "presence": {"status": "invisible", "since": 0, "activities": [], "afk": False}
                        }
                    }
                    await ws.send(json.dumps(identify_payload))

                    # Start Heartbeat Task
                    heartbeat_task = asyncio.create_task(self._heartbeat_loop(ws, heartbeat_interval))

                    try:
                        async for msg in ws:
                            payload = json.loads(msg)
                            op = payload.get("op")
                            event_type = payload.get("t")
                            data = payload.get("d")

                            if op == 0 and event_type == "MESSAGE_CREATE":
                                await self._handle_message_create(token, data)
                    finally:
                        heartbeat_task.cancel()

            except asyncio.CancelledError:
                logger.info(f"Gateway session for {token[:8]} cancelled.")
                break
            except Exception as e:
                logger.warning(f"Gateway connection error for {token[:8]}: {e}. Retrying in {retry_delay}s...")
                await asyncio.sleep(retry_delay)
                retry_delay = min(retry_delay * 2, 60)

    async def _heartbeat_loop(self, ws, interval: float):
        while True:
            await asyncio.sleep(interval)
            try:
                await ws.send(json.dumps({"op": 1, "d": None}))
            except Exception:
                break

    async def _handle_message_create(self, token: str, message_data: dict):
        author = message_data.get("author", {})
        content = message_data.get("content", "")
        channel_id = str(message_data.get("channel_id", ""))
        author_name = author.get("global_name") or author.get("username") or "Discord User"
        author_avatar = author.get("avatar")
        author_id = author.get("id")

        avatar_url = None
        if author_id and author_avatar:
            avatar_url = f"https://cdn.discordapp.com/avatars/{author_id}/{author_avatar}.png"

        # Lookup FCM tokens for this Discord token
        conn = sqlite3.connect(DB_FILE)
        cursor = conn.cursor()
        cursor.execute("SELECT fcm_token FROM user_devices WHERE discord_token = ?", (token,))
        rows = cursor.fetchall()
        conn.close()

        fcm_tokens = [row[0] for row in rows]
        for fcm_token in fcm_tokens:
            await send_fcm_push(
                fcm_token=fcm_token,
                title=author_name,
                body=content,
                channel_id=channel_id,
                avatar_url=avatar_url
            )

session_manager = UserSessionManager()

async def send_fcm_push(fcm_token: str, title: str, body: str, channel_id: str, avatar_url: Optional[str]):
    """Dispatches FCM notification to Google FCM API."""
    logger.info(f"Sending FCM push to token {fcm_token[:10]}... Channel: {channel_id}, Content: {body[:30]}")
    # Note: Replace with Firebase OAuth2 token retrieval in production when firebase-service-account.json is configured
    payload = {
        "to": fcm_token,
        "priority": "high",
        "data": {
            "channelId": channel_id,
            "authorName": title,
            "content": body,
            "avatarUrl": avatar_url or ""
        }
    }
    # Send HTTP POST to FCM legacy / HTTP v1 endpoint
    try:
        async with httpx.AsyncClient() as client:
            res = await client.post("https://fcm.googleapis.com/fcm/send", json=payload, timeout=5.0)
            logger.info(f"FCM Push Response: {res.status_code}")
    except Exception as e:
        logger.error(f"FCM Push Error: {e}")

@app.post("/api/register")
async def register_device(req: RegisterDeviceRequest, background_tasks: BackgroundTasks):
    conn = sqlite3.connect(DB_FILE)
    cursor = conn.cursor()
    cursor.execute("""
        INSERT OR REPLACE INTO user_devices (user_id, discord_token, fcm_token, device_id)
        VALUES (?, ?, ?, ?)
    """, (req.user_id, req.discord_token, req.fcm_token, req.device_id))
    conn.commit()
    conn.close()

    # Start gateway session for user
    background_tasks.add_task(session_manager.start_user_session, req.discord_token)
    return {"status": "success", "message": "Device registered and push session active"}

@app.post("/api/unregister")
async def unregister_device(req: UnregisterDeviceRequest):
    conn = sqlite3.connect(DB_FILE)
    cursor = conn.cursor()
    cursor.execute("DELETE FROM user_devices WHERE discord_token = ? AND fcm_token = ?", (req.discord_token, req.fcm_token))
    conn.commit()
    
    # Check if user has remaining devices
    cursor.execute("SELECT COUNT(*) FROM user_devices WHERE discord_token = ?", (req.discord_token,))
    count = cursor.fetchone()[0]
    conn.close()

    if count == 0:
        await session_manager.stop_user_session(req.discord_token)

    return {"status": "success", "message": "Device unregistered"}

@app.get("/health")
async def health_check():
    return {"status": "ok", "active_sessions": len(session_manager.active_tasks)}

@app.on_event("startup")
async def on_startup():
    # Restore active gateway connections for all registered tokens in DB
    conn = sqlite3.connect(DB_FILE)
    cursor = conn.cursor()
    cursor.execute("SELECT DISTINCT discord_token FROM user_devices")
    tokens = [row[0] for row in cursor.fetchall()]
    conn.close()

    for token in tokens:
        await session_manager.start_user_session(token)
    logger.info(f"Restored {len(tokens)} active user push sessions on startup.")

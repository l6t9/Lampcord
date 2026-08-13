# Deploying Lampcord Multi-User Push Relay on Alwaysdata.com

This guide provides simple step-by-step instructions to host the Lampcord FCM Push Relay server for free on [Alwaysdata.com](https://www.alwaysdata.com/).

---

## Step 1: Create a Free Alwaysdata Account
1. Go to [https://www.alwaysdata.com](https://www.alwaysdata.com) and create a free account.
2. Log into the Alwaysdata Admin Panel.

---

## Step 2: Upload Server Files
1. Go to **Files** in the left sidebar.
2. Upload the `server/` directory containing:
   - `discord_fcm_relay.py`
   - `requirements.txt`

---

## Step 3: Configure Environment & Install Dependencies
1. Go to **SSH** in the left sidebar and enable SSH access.
2. Open SSH terminal or connect via command line:
   ```bash
   ssh your_username@ssh-your_account.alwaysdata.net
   ```
3. Navigate to your app directory and install Python dependencies:
   ```bash
   pip install -r requirements.txt
   ```

---

## Step 4: Add Web Service Site
1. In the Alwaysdata Admin Panel, go to **Web** -> **Sites**.
2. Click **Add a site**:
   - **Name**: `lampcord-push`
   - **Type**: `Python WSGI / ASGI`
   - **Application path**: `/home/your_username/server/discord_fcm_relay.py`
   - **WSGI / ASGI module**: `app` (FastAPI instance)
   - **Working directory**: `/home/your_username/server`
3. Click **Submit**.

---

## Step 5: Test Endpoint
Test your live Alwaysdata push URL in your browser or terminal:
```bash
curl https://your_username.alwaysdata.net/health
```
Expected output:
```json
{"status": "ok", "active_sessions": 0}
```

Once running, set your Alwaysdata server URL in Lampcord settings!

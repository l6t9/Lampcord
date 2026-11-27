; Per-user installer for the Windows build (NSIS 3). No administrator rights: it installs where the user can write, by default
; %LOCALAPPDATA%\<PRODUCT_DIR>, and in-app updates (requiem_updater::apply) keep working there because the folder stays writable.
;
; Build (CI does this in release.yml):
;   makensis /DVERSION=0.1.0 /DSOURCE_DIR=<folder with the exe and its DLLs> /DOUTFILE=<setup.exe> installer\installer.nsi
; Optional: /DVERSION_NUMERIC=0.1.0.0 (four numbers, for the file's version info), and the three names below.
;
; The product's name is the three defines below and nothing else in this file: renaming it later is one edit (or three /D flags).
; Silent use: setup.exe /S /D=C:\Where\To\Install   and   Uninstall.exe /S   (/PURGE also removes the user's data folder).
;
; The user's data lives in <install>\data (requiem_db::paths). Uninstalling asks whether to keep it; a silent uninstall keeps it.

Unicode true

!ifndef PRODUCT_NAME
  !define PRODUCT_NAME "Requiem"
!endif
!ifndef PRODUCT_DIR
  !define PRODUCT_DIR "Requiem"
!endif
!ifndef EXE_NAME
  !define EXE_NAME "requiem.exe"
!endif
!ifndef VERSION
  !define VERSION "0.0.0"
!endif
!ifndef VERSION_NUMERIC
  !define VERSION_NUMERIC "0.0.0.0"
!endif
!ifndef SOURCE_DIR
  !error "pass /DSOURCE_DIR=<folder with the exe and its DLLs>"
!endif
!ifndef OUTFILE
  !define OUTFILE "setup.exe"
!endif

; The toast identity the app registers for itself (notifications.rs); removed when the user's data is.
!define APP_USER_MODEL_KEY "Software\Classes\AppUserModelId\Requiem.Client"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_DIR}"
; Written into the install folder: the uninstaller only empties a folder this installer made.
!define MARKER ".installed-by-setup"

Name "${PRODUCT_NAME}"
OutFile "${OUTFILE}"
RequestExecutionLevel user
InstallDir "$LOCALAPPDATA\${PRODUCT_DIR}"
InstallDirRegKey HKCU "${UNINSTALL_KEY}" "InstallLocation"
SetCompressor /SOLID lzma
BrandingText " "

VIProductVersion "${VERSION_NUMERIC}"
VIAddVersionKey "ProductName" "${PRODUCT_NAME}"
VIAddVersionKey "FileDescription" "${PRODUCT_NAME} setup"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "ProductVersion" "${VERSION}"
VIAddVersionKey "LegalCopyright" "MIT License"

!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "LogicLib.nsh"

!define MUI_ABORTWARNING
!define MUI_DIRECTORYPAGE_TEXT_TOP "Setup will install ${PRODUCT_NAME} in the folder below. Choose a folder you can write to: the app updates itself there, so it must not be under Program Files. A folder called ${PRODUCT_DIR} is made inside the one you pick."
!define MUI_PAGE_CUSTOMFUNCTION_LEAVE DirLeave
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\${EXE_NAME}"
!define MUI_FINISHPAGE_RUN_TEXT "Start ${PRODUCT_NAME}"
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

; Whether $INSTDIR starts with $0 (a folder that needs administrator rights).
!macro StartsWith Result Folder
  StrLen $R8 "${Folder}"
  StrCpy $R9 $INSTDIR $R8
  ${If} $R9 == "${Folder}"
    StrCpy ${Result} 1
  ${Else}
    StrCpy ${Result} 0
  ${EndIf}
!macroend

; Next stays disabled for folders only an administrator can write to.
Function .onVerifyInstDir
  !insertmacro StartsWith $0 "$PROGRAMFILES"
  ${If} $0 == 1
    Abort
  ${EndIf}
  !insertmacro StartsWith $0 "$PROGRAMFILES64"
  ${If} $0 == 1
    Abort
  ${EndIf}
  !insertmacro StartsWith $0 "$WINDIR"
  ${If} $0 == 1
    Abort
  ${EndIf}
FunctionEnd

; The app always gets a folder of its own, so uninstalling can never reach what the user keeps next to it.
Function DirLeave
  StrCpy $0 $INSTDIR 1 -1
  ${If} $0 == "\"
    StrCpy $INSTDIR $INSTDIR -1
  ${EndIf}
  ${GetFileName} $INSTDIR $0
  ${If} $0 != "${PRODUCT_DIR}"
    StrCpy $INSTDIR "$INSTDIR\${PRODUCT_DIR}"
  ${EndIf}
FunctionEnd

Section "Install"
  ; A running copy keeps its exe locked: ask for it to be closed rather than half-installing.
  try_again:
  ClearErrors
  ${If} ${FileExists} "$INSTDIR\${EXE_NAME}"
    Delete "$INSTDIR\${EXE_NAME}"
    ${If} ${Errors}
      ${If} ${Silent}
        Abort "${PRODUCT_NAME} is running. Close it and run setup again."
      ${EndIf}
      MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "${PRODUCT_NAME} is running. Close it, then choose Retry." IDRETRY try_again
      Abort
    ${EndIf}
  ${EndIf}

  ClearErrors
  CreateDirectory "$INSTDIR"
  FileOpen $0 "$INSTDIR\${MARKER}" w
  ${If} ${Errors}
    Abort "Cannot write to $INSTDIR. Choose a folder you can write to."
  ${EndIf}
  FileWrite $0 "${VERSION}"
  FileClose $0

  SetOutPath "$INSTDIR"
  File /r "${SOURCE_DIR}\*.*"
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  CreateShortCut "$SMPROGRAMS\${PRODUCT_NAME}.lnk" "$INSTDIR\${EXE_NAME}"

  ; Apps & features (per user).
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "${PRODUCT_NAME}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\${EXE_NAME}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegStr HKCU "${UNINSTALL_KEY}" "QuietUninstallString" '"$INSTDIR\Uninstall.exe" /S'
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1
  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "EstimatedSize" $0
SectionEnd

Section "Uninstall"
  ; Keep the user's settings and login by default; a silent uninstall keeps them unless /PURGE is given.
  StrCpy $R0 1
  ${If} ${FileExists} "$INSTDIR\data\*.*"
    ${GetParameters} $R1
    ${GetOptions} $R1 "/PURGE" $R2
    ${If} ${Silent}
      ${IfNot} ${Errors}
        StrCpy $R0 0
      ${EndIf}
    ${Else}
      MessageBox MB_YESNO|MB_ICONQUESTION "Keep your settings and login data in $INSTDIR\data?$\r$\n$\r$\nChoose No to remove them too." IDYES keep IDNO drop
      drop:
      StrCpy $R0 0
      keep:
    ${EndIf}
  ${EndIf}

  ; The exe is locked while the app runs.
  uninstall_again:
  ClearErrors
  ${If} ${FileExists} "$INSTDIR\${EXE_NAME}"
    Delete "$INSTDIR\${EXE_NAME}"
    ${If} ${Errors}
      ${If} ${Silent}
        Abort "${PRODUCT_NAME} is running. Close it and run the uninstaller again."
      ${EndIf}
      MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "${PRODUCT_NAME} is running. Close it, then choose Retry." IDRETRY uninstall_again
      Abort
    ${EndIf}
  ${EndIf}

  Delete "$SMPROGRAMS\${PRODUCT_NAME}.lnk"
  DeleteRegKey HKCU "${UNINSTALL_KEY}"

  ${If} ${FileExists} "$INSTDIR\${MARKER}"
    ; Everything the installer and the updater put there; "data" only when asked.
    ${If} $R0 == 0
      RMDir /r "$INSTDIR"
      DeleteRegKey HKCU "${APP_USER_MODEL_KEY}"
    ${Else}
      FindFirst $0 $1 "$INSTDIR\*.*"
      loop:
        StrCmp $1 "" done
        StrCmp $1 "." next
        StrCmp $1 ".." next
        StrCmp $1 "data" next
        ${If} ${FileExists} "$INSTDIR\$1\*.*"
          RMDir /r "$INSTDIR\$1"
        ${Else}
          Delete "$INSTDIR\$1"
        ${EndIf}
      next:
        FindNext $0 $1
        Goto loop
      done:
      FindClose $0
      RMDir "$INSTDIR"
    ${EndIf}
  ${Else}
    ; Not a folder this installer made: take only our own files.
    Delete "$INSTDIR\Uninstall.exe"
  ${EndIf}
SectionEnd

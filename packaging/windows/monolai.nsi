; NSIS Script for Monolai Windows Installer
; Supports x86_64 and arm64 builds

Unicode True

!ifndef VERSION
  !define VERSION "0.1.0"
!endif

!ifndef ARCH
  !define ARCH "x86_64"
!endif

!ifndef BIN_DIR
  !define BIN_DIR "..\..\target\release"
!endif

!ifndef ASSETS_DIR
  !define ASSETS_DIR "assets"
!endif

!ifndef OUT_DIR
  !define OUT_DIR "..\..\dist"
!endif

Name "Monolai"
OutFile "${OUT_DIR}\monolai-v${VERSION}-windows-${ARCH}-installer.exe"
InstallDir "$LOCALAPPDATA\Programs\Monolai"
InstallDirRegKey HKCU "Software\Monolai" "Install_Dir"
RequestExecutionLevel user

; Includes
!include "MUI2.nsh"
!include "FileFunc.nsh"

; MUI Settings
!define MUI_ABORTWARNING
!define MUI_ICON "${ASSETS_DIR}\monolai.ico"
!define MUI_UNICON "${ASSETS_DIR}\monolai.ico"

; Installer Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES

; Finish Page options
!define MUI_FINISHPAGE_RUN "$INSTDIR\monolai-gui.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Launch Monolai now"
!insertmacro MUI_PAGE_FINISH

; Uninstaller Pages
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "Portuguese"

Section "Monolai Application" SecCore
  SetOutPath "$INSTDIR"
  
  ; Write installed files
  File "${BIN_DIR}\monolai-gui.exe"
  File "${BIN_DIR}\monolai.exe"
  File "${ASSETS_DIR}\icons\monolai.ico"

  ; Create uninstaller
  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; Save install location
  WriteRegStr HKCU "Software\Monolai" "Install_Dir" "$INSTDIR"

  ; Add to Add/Remove Programs
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai" "DisplayName" "Monolai"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai" "DisplayIcon" "$INSTDIR\monolai.ico"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai" "Publisher" "Monolai"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai" "URLInfoAbout" "https://github.com/odevsa/monolai"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai" "NoModify" 1
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai" "NoRepair" 1

  ; Calculate and write EstimatedSize
  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai" "EstimatedSize" "$0"

  ; Start Menu Shortcuts
  CreateDirectory "$SMPROGRAMS\Monolai"
  CreateShortcut "$SMPROGRAMS\Monolai\Monolai.lnk" "$INSTDIR\monolai-gui.exe" "" "$INSTDIR\monolai.ico" 0
  CreateShortcut "$SMPROGRAMS\Monolai\Uninstall Monolai.lnk" "$INSTDIR\uninstall.exe" "" "$INSTDIR\uninstall.exe" 0

  ; Desktop Shortcut
  CreateShortcut "$DESKTOP\Monolai.lnk" "$INSTDIR\monolai-gui.exe" "" "$INSTDIR\monolai.ico" 0
SectionEnd

Section "Uninstall"
  ; Remove shortcuts
  Delete "$DESKTOP\Monolai.lnk"
  Delete "$SMPROGRAMS\Monolai\Monolai.lnk"
  Delete "$SMPROGRAMS\Monolai\Uninstall Monolai.lnk"
  RMDir "$SMPROGRAMS\Monolai"

  ; Remove files
  Delete "$INSTDIR\monolai-gui.exe"
  Delete "$INSTDIR\monolai.exe"
  Delete "$INSTDIR\monolai.ico"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"

  ; Remove registry keys
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai"
  DeleteRegKey HKCU "Software\Monolai"
SectionEnd

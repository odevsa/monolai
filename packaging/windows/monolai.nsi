; NSIS Script for Monolai Windows Installer
; Supports x86_64 and arm64 builds

Unicode True

!ifndef VERSION
  !define VERSION "0.3.0"
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
OutFile "${OUT_DIR}\monolai-${VERSION}-windows-${ARCH}.exe"
InstallDir "$PROGRAMFILES64\Monolai"
InstallDirRegKey HKLM "Software\Monolai" "Install_Dir"
RequestExecutionLevel admin

; Includes
!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"

; MUI Settings
!define MUI_ABORTWARNING
!define MUI_ICON "${ASSETS_DIR}\monolai.ico"
!define MUI_UNICON "${ASSETS_DIR}\monolai.ico"

; Installer Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_INSTFILES

; Finish Page options
!define MUI_FINISHPAGE_RUN "$INSTDIR\monolai-gui.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Launch Monolai now"
!insertmacro MUI_PAGE_FINISH

; Uninstaller Pages
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "PortugueseBR"
!insertmacro MUI_LANGUAGE "Spanish"
!insertmacro MUI_LANGUAGE "French"
!insertmacro MUI_LANGUAGE "German"
!insertmacro MUI_LANGUAGE "SimpChinese"
!insertmacro MUI_LANGUAGE "Japanese"
!insertmacro MUI_LANGUAGE "Russian"
!insertmacro MUI_LANGUAGE "Italian"

; Language Strings (English)
LangString SEC_CORE ${LANG_ENGLISH} "Monolai Application (required)"
LangString SEC_GROUP_SHORTCUTS ${LANG_ENGLISH} "Shortcuts"
LangString SEC_STARTMENU ${LANG_ENGLISH} "Start Menu Shortcut"
LangString SEC_DESKTOP ${LANG_ENGLISH} "Desktop Shortcut"
LangString DESC_SecCore ${LANG_ENGLISH} "Core application binaries and required assets."
LangString DESC_SecGroupShortcuts ${LANG_ENGLISH} "Create shortcuts to easily launch Monolai."
LangString DESC_SecStartMenu ${LANG_ENGLISH} "Add Monolai shortcuts to the Windows Start Menu."
LangString DESC_SecDesktop ${LANG_ENGLISH} "Add a shortcut to launch Monolai directly from your Desktop."

; Language Strings (PortugueseBR)
LangString SEC_CORE ${LANG_PORTUGUESEBR} "Aplicativo Monolai (obrigatório)"
LangString SEC_GROUP_SHORTCUTS ${LANG_PORTUGUESEBR} "Atalhos"
LangString SEC_STARTMENU ${LANG_PORTUGUESEBR} "Atalho no Menu Iniciar"
LangString SEC_DESKTOP ${LANG_PORTUGUESEBR} "Atalho na Área de Trabalho"
LangString DESC_SecCore ${LANG_PORTUGUESEBR} "Binários principais do aplicativo e recursos necessários."
LangString DESC_SecGroupShortcuts ${LANG_PORTUGUESEBR} "Criar atalhos para iniciar o Monolai facilmente."
LangString DESC_SecStartMenu ${LANG_PORTUGUESEBR} "Adicionar atalhos do Monolai ao Menu Iniciar do Windows."
LangString DESC_SecDesktop ${LANG_PORTUGUESEBR} "Adicionar um atalho para abrir o Monolai diretamente da Área de Trabalho."

; Language Strings (Spanish)
LangString SEC_CORE ${LANG_SPANISH} "Aplicación Monolai (requerido)"
LangString SEC_GROUP_SHORTCUTS ${LANG_SPANISH} "Accesos directos"
LangString SEC_STARTMENU ${LANG_SPANISH} "Acceso directo en el Menú Inicio"
LangString SEC_DESKTOP ${LANG_SPANISH} "Acceso directo en el Escritorio"
LangString DESC_SecCore ${LANG_SPANISH} "Binarios principales de la aplicación y recursos necesarios."
LangString DESC_SecGroupShortcuts ${LANG_SPANISH} "Crear accesos directos para iniciar Monolai fácilmente."
LangString DESC_SecStartMenu ${LANG_SPANISH} "Agregar accesos directos de Monolai al Menú Inicio de Windows."
LangString DESC_SecDesktop ${LANG_SPANISH} "Agregar un acceso directo para iniciar Monolai directamente desde el Escritorio."

; Language Strings (French)
LangString SEC_CORE ${LANG_FRENCH} "Application Monolai (requis)"
LangString SEC_GROUP_SHORTCUTS ${LANG_FRENCH} "Raccourcis"
LangString SEC_STARTMENU ${LANG_FRENCH} "Raccourci dans le Menu Démarrer"
LangString SEC_DESKTOP ${LANG_FRENCH} "Raccourci sur le Bureau"
LangString DESC_SecCore ${LANG_FRENCH} "Fichiers binaires principaux de l'application et ressources requises."
LangString DESC_SecGroupShortcuts ${LANG_FRENCH} "Créer des raccourcis pour lancer facilement Monolai."
LangString DESC_SecStartMenu ${LANG_FRENCH} "Ajouter des raccourcis Monolai au Menu Démarrer de Windows."
LangString DESC_SecDesktop ${LANG_FRENCH} "Ajouter un raccourci pour lancer Monolai directement depuis le Bureau."

; Language Strings (German)
LangString SEC_CORE ${LANG_GERMAN} "Monolai-Anwendung (erforderlich)"
LangString SEC_GROUP_SHORTCUTS ${LANG_GERMAN} "Verknüpfungen"
LangString SEC_STARTMENU ${LANG_GERMAN} "Verknüpfung im Startmenü"
LangString SEC_DESKTOP ${LANG_GERMAN} "Desktop-Verknüpfung"
LangString DESC_SecCore ${LANG_GERMAN} "Hauptprogrammdateien und erforderliche Ressourcen."
LangString DESC_SecGroupShortcuts ${LANG_GERMAN} "Verknüpfungen erstellen, um Monolai einfach zu starten."
LangString DESC_SecStartMenu ${LANG_GERMAN} "Monolai-Verknüpfungen zum Windows-Startmenü hinzufügen."
LangString DESC_SecDesktop ${LANG_GERMAN} "Eine Verknüpfung auf dem Desktop hinzufügen, um Monolai direkt zu starten."

; Language Strings (SimpChinese)
LangString SEC_CORE ${LANG_SIMPCHINESE} "Monolai 应用程序 (必选)"
LangString SEC_GROUP_SHORTCUTS ${LANG_SIMPCHINESE} "快捷方式"
LangString SEC_STARTMENU ${LANG_SIMPCHINESE} "开始菜单快捷方式"
LangString SEC_DESKTOP ${LANG_SIMPCHINESE} "桌面快捷方式"
LangString DESC_SecCore ${LANG_SIMPCHINESE} "核心应用程序二进制文件及必需资源。"
LangString DESC_SecGroupShortcuts ${LANG_SIMPCHINESE} "创建快捷方式以便快速启动 Monolai。"
LangString DESC_SecStartMenu ${LANG_SIMPCHINESE} "将 Monolai 快捷方式添加到 Windows 开始菜单。"
LangString DESC_SecDesktop ${LANG_SIMPCHINESE} "在桌面添加直接启动 Monolai 的快捷方式。"

; Language Strings (Japanese)
LangString SEC_CORE ${LANG_JAPANESE} "Monolai アプリケーション (必須)"
LangString SEC_GROUP_SHORTCUTS ${LANG_JAPANESE} "ショートカット"
LangString SEC_STARTMENU ${LANG_JAPANESE} "スタートメニューのショートカット"
LangString SEC_DESKTOP ${LANG_JAPANESE} "デスクトップのショートカット"
LangString DESC_SecCore ${LANG_JAPANESE} "アプリケーション本体および必要なリソース。"
LangString DESC_SecGroupShortcuts ${LANG_JAPANESE} "Monolai を簡単に起動するためのショートカットを作成します。"
LangString DESC_SecStartMenu ${LANG_JAPANESE} "Windows スタートメニューに Monolai のショートカットを追加します。"
LangString DESC_SecDesktop ${LANG_JAPANESE} "デスクトップに Monolai を直接起動するショートカットを追加します。"

; Language Strings (Russian)
LangString SEC_CORE ${LANG_RUSSIAN} "Приложение Monolai (обязательно)"
LangString SEC_GROUP_SHORTCUTS ${LANG_RUSSIAN} "Ярлыки"
LangString SEC_STARTMENU ${LANG_RUSSIAN} "Ярлык в меню «Пуск»"
LangString SEC_DESKTOP ${LANG_RUSSIAN} "Ярлык на Рабочем столе"
LangString DESC_SecCore ${LANG_RUSSIAN} "Основные исполняемые файлы приложения и необходимые ресурсы."
LangString DESC_SecGroupShortcuts ${LANG_RUSSIAN} "Создать ярлыки для быстрого запуска Monolai."
LangString DESC_SecStartMenu ${LANG_RUSSIAN} "Добавить ярлыки Monolai в меню «Пуск» Windows."
LangString DESC_SecDesktop ${LANG_RUSSIAN} "Добавить ярлык на Рабочий стол для быстрого запуска Monolai."

; Language Strings (Italian)
LangString SEC_CORE ${LANG_ITALIAN} "Applicazione Monolai (obbligatorio)"
LangString SEC_GROUP_SHORTCUTS ${LANG_ITALIAN} "Collegamenti"
LangString SEC_STARTMENU ${LANG_ITALIAN} "Collegamento nel Menu Start"
LangString SEC_DESKTOP ${LANG_ITALIAN} "Collegamento sul Desktop"
LangString DESC_SecCore ${LANG_ITALIAN} "File binari principali dell'applicazione e risorse necessarie."
LangString DESC_SecGroupShortcuts ${LANG_ITALIAN} "Crea collegamenti per avviare facilmente Monolai."
LangString DESC_SecStartMenu ${LANG_ITALIAN} "Aggiungi i collegamenti di Monolai al Menu Start di Windows."
LangString DESC_SecDesktop ${LANG_ITALIAN} "Aggiungi un collegamento sul Desktop per avviare direttamente Monolai."

; Macro to terminate any running Monolai processes (GUI and CLI backend)
; Works in both installer and uninstaller contexts.
!macro CloseMonolaiProcessesHelper UN
Function ${UN}CloseMonolaiProcesses
  DetailPrint "Closing running Monolai instances..."
  ${If} ${RunningX64}
    ${DisableX64FSRedirection}
    nsExec::Exec '"$SYSDIR\taskkill.exe" /F /T /IM "monolai-gui.exe"'
    Pop $0
    nsExec::Exec '"$SYSDIR\taskkill.exe" /F /T /IM "monolai.exe"'
    Pop $0
    ${EnableX64FSRedirection}
  ${Else}
    nsExec::Exec '"$SYSDIR\taskkill.exe" /F /T /IM "monolai-gui.exe"'
    Pop $0
    nsExec::Exec '"$SYSDIR\taskkill.exe" /F /T /IM "monolai.exe"'
    Pop $0
  ${EndIf}
  Sleep 1000
FunctionEnd
!macroend

!insertmacro CloseMonolaiProcessesHelper ""
!insertmacro CloseMonolaiProcessesHelper "un."

Section $(SEC_CORE) SecCore
  SectionIn RO
  SetShellVarContext all
  SetOutPath "$INSTDIR"
  
  ; Terminate running processes before installing new files
  Call CloseMonolaiProcesses

  ; Write installed files
  File "${BIN_DIR}\monolai-gui.exe"
  File "${BIN_DIR}\monolai.exe"
  File "${ASSETS_DIR}\monolai.ico"

  ; Create uninstaller
  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; Save install location
  WriteRegStr HKLM "Software\Monolai" "Install_Dir" "$INSTDIR"

  ; Add to Add/Remove Programs
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai" "DisplayName" "Monolai"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai" "DisplayIcon" "$INSTDIR\monolai.ico"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai" "Publisher" "Monolai"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai" "URLInfoAbout" "https://github.com/odevsa/monolai"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai" "NoModify" 1
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai" "NoRepair" 1

  ; Clean up any obsolete HKLM Run value to prevent duplicate startup
  DeleteRegValue HKLM "Software\Microsoft\Windows\CurrentVersion\Run" "Monolai"
SectionEnd

SectionGroup /e $(SEC_GROUP_SHORTCUTS) SecGroupShortcuts
  Section $(SEC_STARTMENU) SecStartMenu
    SetShellVarContext all
    CreateDirectory "$SMPROGRAMS\Monolai"
    CreateShortcut "$SMPROGRAMS\Monolai\Monolai.lnk" "$INSTDIR\monolai-gui.exe" "" "$INSTDIR\monolai.ico" 0
    CreateShortcut "$SMPROGRAMS\Monolai\Uninstall Monolai.lnk" "$INSTDIR\uninstall.exe" "" "$INSTDIR\uninstall.exe" 0
  SectionEnd

  Section $(SEC_DESKTOP) SecDesktop
    SetShellVarContext all
    CreateShortcut "$DESKTOP\Monolai.lnk" "$INSTDIR\monolai-gui.exe" "" "$INSTDIR\monolai.ico" 0
  SectionEnd
SectionGroupEnd

Section "-Post"
  ; Calculate and write EstimatedSize
  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai" "EstimatedSize" "$0"
SectionEnd

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SecCore} $(DESC_SecCore)
  !insertmacro MUI_DESCRIPTION_TEXT ${SecGroupShortcuts} $(DESC_SecGroupShortcuts)
  !insertmacro MUI_DESCRIPTION_TEXT ${SecStartMenu} $(DESC_SecStartMenu)
  !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} $(DESC_SecDesktop)
!insertmacro MUI_FUNCTION_DESCRIPTION_END

Section "Uninstall"
  SetShellVarContext all

  ; Terminate running processes before removing files
  Call un.CloseMonolaiProcesses

  ; Clean up autostart registry entries
  DeleteRegValue HKLM "Software\Microsoft\Windows\CurrentVersion\Run" "Monolai"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Monolai"

  ; Remove shortcuts (if present)
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
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Monolai"
  DeleteRegKey HKLM "Software\Monolai"
SectionEnd

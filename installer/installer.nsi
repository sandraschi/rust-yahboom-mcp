; Boomy Control (Rust) v0.1.0 - current-user installer (no admin, no UAC).
; Built with NSIS 3 (makensis installer/installer.nsi, CWD = repo root).
; Installs the egui dashboard + the CLI/MCP server side by side.

!include "MUI2.nsh"

Name "Boomy Control (Rust) 0.1.0"
OutFile "..\target\release\bundle\Boomy-Control-Rust-0.1.0-setup.exe"
InstallDir "$LOCALAPPDATA\Programs\rust-yahboom-mcp"
RequestExecutionLevel user

!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "Binaries" SecBinaries
  SetOutPath "$INSTDIR"
  File "..\target\release\yahboom-app.exe"
  File "..\target\release\rust-yahboom-mcp.exe"
  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\rust-yahboom-mcp" \
    "DisplayName" "Boomy Control (Rust) 0.1.0"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\rust-yahboom-mcp" \
    "UninstallString" "$INSTDIR\uninstall.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\rust-yahboom-mcp" \
    "DisplayIcon" "$INSTDIR\yahboom-app.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\rust-yahboom-mcp" \
    "DisplayVersion" "0.1.0"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\rust-yahboom-mcp" \
    "Publisher" "sandraschi"
SectionEnd

Section "Start Menu shortcuts" SecShortcuts
  CreateDirectory "$SMPROGRAMS\Boomy Control (Rust)"
  CreateShortcut "$SMPROGRAMS\Boomy Control (Rust)\Boomy Control (Rust).lnk" "$INSTDIR\yahboom-app.exe"
  CreateShortcut "$SMPROGRAMS\Boomy Control (Rust)\Uninstall.lnk" "$INSTDIR\uninstall.exe"
SectionEnd

Section "uninstall"
  Delete "$INSTDIR\yahboom-app.exe"
  Delete "$INSTDIR\rust-yahboom-mcp.exe"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  Delete "$SMPROGRAMS\Boomy Control (Rust)\Boomy Control (Rust).lnk"
  Delete "$SMPROGRAMS\Boomy Control (Rust)\Uninstall.lnk"
  RMDir "$SMPROGRAMS\Boomy Control (Rust)"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\rust-yahboom-mcp"
SectionEnd

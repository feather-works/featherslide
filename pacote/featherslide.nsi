; Instalador do FeatherSlide para Windows (NSIS). Gerado por pacote/gerar.sh:
;   makensis -DVERSAO=0.0.1 -DEXE=caminho/featherslide.exe -DSAIDA=caminho/instalador.exe pacote/featherslide.nsi
; Instala por usuário (sem pedir administrador), cria atalhos, associa o .fsdz
; e registra o desinstalador em "Aplicativos instalados".

Unicode true
!include "MUI2.nsh"

!ifndef VERSAO
  !define VERSAO "0.0.0"
!endif

Name "FeatherSlide ${VERSAO}"
OutFile "${SAIDA}"
InstallDir "$LOCALAPPDATA\Programs\FeatherSlide"
InstallDirRegKey HKCU "Software\FeatherWorks\FeatherSlide" "Pasta"
RequestExecutionLevel user
SetCompressor /SOLID lzma

!define MUI_ICON "icones\featherslide.ico"
!define MUI_UNICON "icones\featherslide.ico"
!define MUI_FINISHPAGE_RUN "$INSTDIR\featherslide.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Abrir o FeatherSlide"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "PortugueseBR"

!define DESINSTALAR "Software\Microsoft\Windows\CurrentVersion\Uninstall\FeatherSlide"

Section "FeatherSlide"
  SetOutPath "$INSTDIR"
  File "/oname=featherslide.exe" "${EXE}"
  File "icones\featherslide.ico"
  File "/oname=LICENSE.txt" "..\LICENSE"
  WriteUninstaller "$INSTDIR\desinstalar.exe"

  CreateShortCut "$SMPROGRAMS\FeatherSlide.lnk" "$INSTDIR\featherslide.exe" "" "$INSTDIR\featherslide.ico"
  CreateShortCut "$DESKTOP\FeatherSlide.lnk" "$INSTDIR\featherslide.exe" "" "$INSTDIR\featherslide.ico"

  ; Apresentação do FeatherSlide (.fsdz): ícone e abrir com dois cliques.
  WriteRegStr HKCU "Software\Classes\.fsdz" "" "FeatherSlide.Apresentacao"
  WriteRegStr HKCU "Software\Classes\FeatherSlide.Apresentacao" "" "Apresentação do FeatherSlide"
  WriteRegStr HKCU "Software\Classes\FeatherSlide.Apresentacao\DefaultIcon" "" "$INSTDIR\featherslide.ico"
  WriteRegStr HKCU "Software\Classes\FeatherSlide.Apresentacao\shell\open\command" "" '"$INSTDIR\featherslide.exe" "%1"'
  ; "Abrir com" para .pptx e .odp, sem tomar o lugar do programa padrão.
  WriteRegStr HKCU "Software\Classes\.pptx\OpenWithProgids" "FeatherSlide.Apresentacao" ""
  WriteRegStr HKCU "Software\Classes\.odp\OpenWithProgids" "FeatherSlide.Apresentacao" ""

  WriteRegStr HKCU "Software\FeatherWorks\FeatherSlide" "Pasta" "$INSTDIR"
  WriteRegStr HKCU "${DESINSTALAR}" "DisplayName" "FeatherSlide"
  WriteRegStr HKCU "${DESINSTALAR}" "DisplayVersion" "${VERSAO}"
  WriteRegStr HKCU "${DESINSTALAR}" "Publisher" "FeatherWorks"
  WriteRegStr HKCU "${DESINSTALAR}" "DisplayIcon" "$INSTDIR\featherslide.ico"
  WriteRegStr HKCU "${DESINSTALAR}" "URLInfoAbout" "https://github.com/feather-works/featherslide"
  WriteRegStr HKCU "${DESINSTALAR}" "UninstallString" '"$INSTDIR\desinstalar.exe"'
  WriteRegDWORD HKCU "${DESINSTALAR}" "NoModify" 1
  WriteRegDWORD HKCU "${DESINSTALAR}" "NoRepair" 1
SectionEnd

Section "Uninstall"
  Delete "$SMPROGRAMS\FeatherSlide.lnk"
  Delete "$DESKTOP\FeatherSlide.lnk"
  Delete "$INSTDIR\featherslide.exe"
  Delete "$INSTDIR\featherslide.ico"
  Delete "$INSTDIR\LICENSE.txt"
  Delete "$INSTDIR\desinstalar.exe"
  RMDir "$INSTDIR"
  DeleteRegKey HKCU "Software\Classes\FeatherSlide.Apresentacao"
  DeleteRegValue HKCU "Software\Classes\.pptx\OpenWithProgids" "FeatherSlide.Apresentacao"
  DeleteRegValue HKCU "Software\Classes\.odp\OpenWithProgids" "FeatherSlide.Apresentacao"
  ReadRegStr $0 HKCU "Software\Classes\.fsdz" ""
  StrCmp $0 "FeatherSlide.Apresentacao" 0 +2
    DeleteRegKey HKCU "Software\Classes\.fsdz"
  DeleteRegKey HKCU "Software\FeatherWorks\FeatherSlide"
  DeleteRegKey HKCU "${DESINSTALAR}"
SectionEnd

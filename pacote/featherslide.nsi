; Instalador do FeatherSlide para Windows (NSIS). Gerado por pacote/gerar.sh:
;   makensis -DVERSAO=0.0.1 -DEXE=caminho/featherslide.exe -DSAIDA=caminho/instalador.exe pacote/featherslide.nsi
; Instala para todos os usuários em "Arquivos de Programas\FeatherWorks\FeatherSlide" (pede
; administrador uma vez: só o administrador altera os .exe; a pasta FeatherWorks é da suíte), cria atalhos, associa o .fsdz
; e registra o desinstalador em "Aplicativos instalados".

Unicode true
!include "MUI2.nsh"

!ifndef VERSAO
  !define VERSAO "0.0.0"
!endif

Name "FeatherSlide ${VERSAO}"
OutFile "${SAIDA}"
InstallDir "$PROGRAMFILES64\FeatherWorks\FeatherSlide"
InstallDirRegKey HKLM "Software\FeatherWorks\FeatherSlide" "Pasta"
RequestExecutionLevel admin
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

!include "FileFunc.nsh"
!define DESINSTALAR "Software\Microsoft\Windows\CurrentVersion\Uninstall\FeatherSlide"

; Atalhos de todos os usuários e o registro de 64 bits (o programa é de 64 bits).
Function .onInit
  SetShellVarContext all
  SetRegView 64
FunctionEnd

Function un.onInit
  SetShellVarContext all
  SetRegView 64
FunctionEnd

Section "FeatherSlide"
  ; As versões antigas instalavam só para o usuário (%LOCALAPPDATA%\Programs): tira essa cópia.
  ; _?= faz o desinstalador antigo rodar no lugar e esperar; o próprio .exe e a pasta saem aqui.
  ReadRegStr $2 HKCU "Software\FeatherWorks\FeatherSlide" "Pasta"
  StrCmp $2 "" antiga_fora
  IfFileExists "$2\desinstalar.exe" 0 antiga_fora
    ExecWait '"$2\desinstalar.exe" /S _?=$2'
    Delete "$2\desinstalar.exe"
    RMDir "$2"
  antiga_fora:

  SetOutPath "$INSTDIR"
  ; Com o programa aberto, o .exe em uso não se sobrescreve, mas se renomeia; o .old sai
  ; na próxima instalação ou na desinstalação.
  Delete "$INSTDIR\featherslide.old"
  Rename "$INSTDIR\featherslide.exe" "$INSTDIR\featherslide.old"
  File "/oname=featherslide.exe" "${EXE}"
  File "icones\featherslide.ico"
  File "/oname=LICENSE.txt" "..\LICENSE"
  WriteUninstaller "$INSTDIR\desinstalar.exe"

  CreateShortCut "$SMPROGRAMS\FeatherSlide.lnk" "$INSTDIR\featherslide.exe" "" "$INSTDIR\featherslide.ico"
  CreateShortCut "$DESKTOP\FeatherSlide.lnk" "$INSTDIR\featherslide.exe" "" "$INSTDIR\featherslide.ico"

  ; Apresentação do FeatherSlide (.fsdz): ícone e abrir com dois cliques.
  WriteRegStr HKLM "Software\Classes\.fsdz" "" "FeatherSlide.Apresentacao"
  WriteRegStr HKLM "Software\Classes\FeatherSlide.Apresentacao" "" "Apresentação do FeatherSlide"
  WriteRegStr HKLM "Software\Classes\FeatherSlide.Apresentacao\DefaultIcon" "" "$INSTDIR\featherslide.ico"
  WriteRegStr HKLM "Software\Classes\FeatherSlide.Apresentacao\shell\open\command" "" '"$INSTDIR\featherslide.exe" "%1"'
  ; "Abrir com" para .pptx e .odp, sem tomar o lugar do programa padrão.
  WriteRegStr HKLM "Software\Classes\.pptx\OpenWithProgids" "FeatherSlide.Apresentacao" ""
  WriteRegStr HKLM "Software\Classes\.odp\OpenWithProgids" "FeatherSlide.Apresentacao" ""

  WriteRegStr HKLM "Software\FeatherWorks\FeatherSlide" "Pasta" "$INSTDIR"
  WriteRegStr HKLM "${DESINSTALAR}" "DisplayName" "FeatherSlide"
  WriteRegStr HKLM "${DESINSTALAR}" "DisplayVersion" "${VERSAO}"
  WriteRegStr HKLM "${DESINSTALAR}" "Publisher" "FeatherWorks"
  ; Tamanho em "Programas e Recursos" (KB, do que foi instalado).
  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD HKLM "${DESINSTALAR}" "EstimatedSize" "$0"
  WriteRegStr HKLM "${DESINSTALAR}" "DisplayIcon" "$INSTDIR\featherslide.ico"
  WriteRegStr HKLM "${DESINSTALAR}" "URLInfoAbout" "https://github.com/feather-works/featherslide"
  WriteRegStr HKLM "${DESINSTALAR}" "UninstallString" '"$INSTDIR\desinstalar.exe"'
  WriteRegStr HKLM "${DESINSTALAR}" "InstallLocation" "$INSTDIR"
  WriteRegDWORD HKLM "${DESINSTALAR}" "NoModify" 1
  WriteRegDWORD HKLM "${DESINSTALAR}" "NoRepair" 1
SectionEnd

Section "Uninstall"
  Delete "$SMPROGRAMS\FeatherSlide.lnk"
  Delete "$DESKTOP\FeatherSlide.lnk"
  Delete "$INSTDIR\featherslide.exe"
  Delete "$INSTDIR\featherslide.old"
  Delete "$INSTDIR\featherslide.ico"
  Delete "$INSTDIR\LICENSE.txt"
  Delete "$INSTDIR\desinstalar.exe"
  RMDir "$INSTDIR"
  ; A pasta da suíte só sai se ficou vazia (sem outro programa nem o Launcher).
  RMDir "$INSTDIR\.."
  DeleteRegKey HKLM "Software\Classes\FeatherSlide.Apresentacao"
  DeleteRegValue HKLM "Software\Classes\.pptx\OpenWithProgids" "FeatherSlide.Apresentacao"
  DeleteRegValue HKLM "Software\Classes\.odp\OpenWithProgids" "FeatherSlide.Apresentacao"
  ReadRegStr $0 HKLM "Software\Classes\.fsdz" ""
  StrCmp $0 "FeatherSlide.Apresentacao" 0 +2
    DeleteRegKey HKLM "Software\Classes\.fsdz"
  DeleteRegKey HKLM "Software\FeatherWorks\FeatherSlide"
  DeleteRegKey /ifempty HKLM "Software\FeatherWorks"
  DeleteRegKey HKLM "${DESINSTALAR}"
SectionEnd

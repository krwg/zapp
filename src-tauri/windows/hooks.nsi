!include nsDialogs.nsh
!include LogicLib.nsh

Var CleanData
Var CleanCheckbox

; Shown first. Skipped when there is nothing left from a previous zapp run.
Page custom PageCleanData PageLeaveCleanData

Function PageCleanData
  IfFileExists "$APPDATA\app.zapp.desktop\*.*" show 0
  IfFileExists "$LOCALAPPDATA\app.zapp.desktop\*.*" show 0
  Abort

  show:
  nsDialogs::Create 1018
  Pop $0
  ${If} $0 == error
    MessageBox MB_ICONEXCLAMATION "Не удалось открыть шаг очистки. Установка продолжится без удаления данных."
    Abort
  ${EndIf}

  ${NSD_CreateLabel} 0 0 100% 48u "Найдены данные прошлой установки zapp.$\r$\nУдалить настройки, логи и скачанный движок zapret?$\r$\nRemove previous settings, logs, and the downloaded zapret engine?"
  Pop $0

  ${NSD_CreateCheckbox} 0 56u 100% 16u "Полная очистка / Full cleanup"
  Pop $CleanCheckbox
  ${NSD_Check} $CleanCheckbox

  nsDialogs::Show
FunctionEnd

Function PageLeaveCleanData
  ${NSD_GetState} $CleanCheckbox $CleanData
FunctionEnd

!macro ZappWipeBody
  nsExec::ExecToLog 'taskkill /F /IM zapp.exe'
  Pop $0
  nsExec::ExecToLog 'taskkill /F /IM winws.exe'
  Pop $0
  Sleep 600

  ClearErrors
  RMDir /r "$APPDATA\app.zapp.desktop"
  ${If} ${Errors}
    MessageBox MB_ICONEXCLAMATION "Не удалось полностью удалить:$\n$APPDATA\app.zapp.desktop$\n$\nЗакройте zapp (в том числе из трея) и удалите папку вручную. Установка продолжится."
    ClearErrors
  ${EndIf}

  ClearErrors
  RMDir /r "$LOCALAPPDATA\app.zapp.desktop"
  ${If} ${Errors}
    MessageBox MB_ICONEXCLAMATION "Не удалось полностью удалить:$\n$LOCALAPPDATA\app.zapp.desktop$\n$\nУстановка продолжится."
    ClearErrors
  ${EndIf}
!macroend

Function CleanZappData
  !insertmacro ZappWipeBody
FunctionEnd

Function un.CleanZappData
  !insertmacro ZappWipeBody
FunctionEnd

!macro NSIS_HOOK_PREINSTALL
  nsExec::ExecToLog 'taskkill /F /IM zapp.exe'
  Pop $0
  Sleep 400
  ${If} $CleanData == ${BST_CHECKED}
    Call CleanZappData
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::ExecToLog 'taskkill /F /IM zapp.exe'
  Pop $0
  nsExec::ExecToLog 'taskkill /F /IM winws.exe'
  Pop $0
  Sleep 400
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  IfFileExists "$APPDATA\app.zapp.desktop\*.*" ask 0
  IfFileExists "$LOCALAPPDATA\app.zapp.desktop\*.*" ask done
  Goto done
  ask:
    MessageBox MB_YESNO|MB_ICONQUESTION "Удалить также данные zapp (настройки и скачанный движок)?" IDYES wipe IDNO done
  wipe:
    Call un.CleanZappData
  done:
!macroend

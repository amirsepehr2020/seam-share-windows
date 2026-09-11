!macro NSIS_HOOK_POSTINSTALL
  nsExec::ExecToLog 'netsh advfirewall firewall add rule name="SEAM Share TCP" dir=in action=allow protocol=TCP localport=38948 profile=private'
  nsExec::ExecToLog 'netsh advfirewall firewall add rule name="SEAM Share UDP Discovery" dir=in action=allow protocol=UDP localport=38947 profile=private'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="SEAM Share TCP"'
  nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="SEAM Share UDP Discovery"'
!macroend

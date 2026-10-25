; NSIS 安装器定制钩子（tauri.conf.json → bundle.windows.nsis.installerHooks）
; 本文件被 include 在 installer.nsi 模板头部，早于 MUI 完成页定义。
; 模板把「创建桌面快捷方式」复用为 MUI 完成页的 SHOWREADME 复选框（默认勾选），
; 提前定义 NOTCHECKED 使其默认不勾选，用户仍可手动勾选创建。
!define MUI_FINISHPAGE_SHOWREADME_NOTCHECKED

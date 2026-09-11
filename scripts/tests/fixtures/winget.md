These three manifests reproduce microsoft/winget-pkgs PR #433145 at commit
f1b722d3d7022b6ff389b8e362c2ae53fec860ad. They intentionally use the published
v0.2.0 Windows ZIP, not an unreleased build. The Test WinGet installation
workflow validates, installs, runs, and uninstalls this package on Windows,
retaining logs and a generated PDF for upstream manual validation.

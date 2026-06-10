[Setup]
AppId={{019EB36D-2D44-7A10-A0D3-1DA29AA7865C}}
AppName=eDirStat
AppVersion={#AppVersion}
AppPublisher=Cody Wyatt Neiman (xangelix)
AppPublisherURL=https://github.com/xangelix/edirstat
AppSupportURL=https://github.com/xangelix/edirstat/issues
AppUpdatesURL=https://github.com/xangelix/edirstat/releases
DefaultDirName={autopf}\eDirStat
DefaultGroupName=eDirStat
DisableProgramGroupPage=yes
LicenseFile=LICENSE
; Output directory and name
OutputDir=staging
OutputBaseFilename=edirstat-setup-x86_64
SetupIconFile=assets\img\icon.ico
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "target\release\edirstat.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\eDirStat"; Filename: "{app}\edirstat.exe"; IconFilename: "{app}\edirstat.exe"
Name: "{autodesktop}\eDirStat"; Filename: "{app}\edirstat.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\edirstat.exe"; Description: "{cm:LaunchProgram,eDirStat}"; Flags: nowait postinstall skipifsilent

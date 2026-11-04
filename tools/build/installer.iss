#ifndef AppVersion
  #error AppVersion is required: ISCC /DAppVersion=X.Y.Z
#endif
#ifndef AppExe
  #error AppExe is required: ISCC /DAppExe=<path to podcast-monitor.exe>
#endif
#define Root "..\.."

[Setup]
AppId={{EE27F7FE-2C77-4D65-9BD0-5B0CD4313DDA}
AppName=Podcast-Monitor
AppVersion={#AppVersion}
AppVerName=Podcast-Monitor {#AppVersion}
AppPublisher=Felipe Carvajal Brown
DefaultDirName={autopf}\Podcast-Monitor
DefaultGroupName=Podcast-Monitor
DisableProgramGroupPage=yes
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir={#Root}\work\installer
OutputBaseFilename=PodcastMonitor-windows-setup
SetupIconFile={#Root}\assets\brand\windows\podcast-monitor.ico
UninstallDisplayIcon={app}\podcast-monitor.ico
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"

[Files]
Source: "{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Root}\work\vulkan-loader\bin\vulkan-1.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Root}\work\vulkan-loader\bin\LICENSE-vulkan-loader.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Root}\assets\brand\windows\podcast-monitor.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Root}\assets\fonts\LICENSE-latin-modern.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Root}\work\redist\vc_redist.x64.exe"; DestDir: "{tmp}"; Flags: deleteafterinstall; Check: VCRuntimeMissing

[Icons]
Name: "{group}\Podcast-Monitor"; Filename: "{app}\podcast-monitor.exe"; IconFilename: "{app}\podcast-monitor.ico"
Name: "{group}\Desinstalar Podcast-Monitor"; Filename: "{uninstallexe}"

[Run]
Filename: "{tmp}\vc_redist.x64.exe"; Parameters: "/install /quiet /norestart"; StatusMsg: "Instalando los componentes de Microsoft Visual C++..."; Check: VCRuntimeMissing; Flags: waituntilterminated
Filename: "{app}\podcast-monitor.exe"; Description: "Abrir Podcast-Monitor"; Flags: nowait postinstall skipifsilent

[Code]
function VCRuntimeMissing: Boolean;
var
  Installed: Cardinal;
begin
  Result := not (RegQueryDWordValue(HKLM64, 'SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64', 'Installed', Installed) and (Installed = 1));
end;

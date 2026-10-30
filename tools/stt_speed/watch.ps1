param(
    [string]$Out = (Join-Path $PSScriptRoot "..\..\work\speed-week"),
    [switch]$Once
)

$bar = "#" * 60
$Host.UI.RawUI.BackgroundColor = "DarkRed"
$Host.UI.RawUI.ForegroundColor = "White"
$Host.UI.RawUI.WindowTitle = "NO CERRAR LA TAPA"

while ($true) {
    Clear-Host
    $progress = Get-Content (Join-Path $Out "progress.json") -Raw -ErrorAction SilentlyContinue | ConvertFrom-Json -ErrorAction SilentlyContinue
    $finished = Test-Path (Join-Path $Out "finished")
    Write-Host ""
    Write-Host "  $bar"
    Write-Host ""
    Write-Host "     N O   C E R R A R   L A   T A P A" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "     N O   D E S E N C H U F A R" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  $bar"
    Write-Host ""
    Write-Host "     El computador esta trabajando solo durante la noche."
    Write-Host "     Se va a dormir solo cuando termine. No hay que tocar nada."
    Write-Host ""
    if ($finished) {
        Write-Host "     Trabajo terminado. Se dormira solo en unos minutos." -ForegroundColor Green
    } elseif ($progress) {
        $percent = [math]::Floor(100 * ($progress.step - 1) / $progress.of)
        Write-Host ("     Avance: paso {0} de {1}  ({2} %)" -f $progress.step, $progress.of, $percent) -ForegroundColor Cyan
        Write-Host ("     Ultima actualizacion: {0}" -f $progress.updated.Substring(11))
    } else {
        Write-Host "     Esperando el primer avance..."
    }
    Write-Host ""
    Write-Host ("     Hora: {0:HH:mm}" -f (Get-Date))
    Write-Host ""
    Write-Host "  $bar"
    if ($Once) { break }
    Start-Sleep -Seconds 30
}

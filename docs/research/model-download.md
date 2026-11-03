# Model download and Windows installer

## The model file

- Source: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin`, which redirects (302) to Hugging Face's CDN.
- Size 1,624,555,275 bytes, SHA-256 `1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69`, from the Hugging Face API (`/api/models/ggerganov/whisper.cpp/tree/main`, `lfs.oid`) and the `X-Linked-Size` and `X-Linked-ETag` headers.
- The copy the app has used since October (`%APPDATA%\Podcast-Monitor\models\ggml-large-v3-turbo.bin`) has the same size and SHA-256 (`sha256sum`), so the pinned hash is the one every measurement so far ran on.

## Chosen by Felipe

- The window shows a notice and a Descargar modelo button when the model is missing; runs stay off until it is there.
- The finished file is checked against the pinned SHA-256; a wrong file is deleted and the user is told to retry.
- An interrupted download resumes from the partial file with an HTTP range request.
- Wording:
  - Notice: "Para transcribir, la app necesita el modelo de voz (1,6 GB). Se descarga una sola vez."
  - Button: "Descargar modelo"
  - Progress: "Descargando el modelo de voz: 640 de 1.549 MB (41 %)"
  - Check: "Revisando que el modelo esté completo…"
  - Done: "Modelo de voz listo."
  - Interrupted: "La descarga se interrumpió. Revisa la conexión a internet y usa Descargar modelo para seguir donde quedó."
  - Bad file: "El modelo descargado llegó dañado y se borró. Usa Descargar modelo para bajarlo de nuevo."
  - Run without model: "Falta el modelo de voz. Usa Descargar modelo y luego Continuar."
- Windows is installed with Inno Setup (exe, Start menu shortcut, uninstaller); the model stays an in-app download, so the Mac build uses the same path.

## Inno Setup

- Installed here: Inno Setup 6 at `C:\Program Files (x86)\Inno Setup 6` (`ISCC.exe`, examples `DownloadFiles.iss` and `CodeDownloadFiles.iss`).
- Licence: use for any purpose, commercial included; copyright notices kept in source and binary redistributions (https://jrsoftware.org/files/is/license.txt). Source public.
- Its download page takes the expected SHA-256 as the third argument of `DownloadPage.Add` and stops with "Invalid file hash" on a mismatch (https://jrsoftware.org/ishelp/topic_isxfunc_createdownloadpage.htm, https://groups.google.com/g/innosetup/c/ZJcIBLvMDuI). No resume of a partial download found in its docs. Windows only.
- Open-source alternatives looked at: NSIS (zlib licence; downloads and hashes through plugins) and WiX (MS-RL; MSI, no built-in download page).

## Built (phase 9 item 1)

- `src/model.rs` downloads to `ggml-large-v3-turbo.bin.part`, resumes with `Range: bytes=<done>-` when the server answers 206, starts over on a 200, checks the SHA-256, deletes a wrong file and renames a good one into place. The window shows the notice and button while the model is missing and keeps Generar informe and Generar informe semanal off until it is there.
- Tests: a download cut at 1,000,000 of 3,000,000 bytes resumes from the partial file; a server ignoring the range restarts the file; a wrong hash deletes it; the window texts and the disabled buttons. 89 passed, 8 ignored.
- Live: the ignored test `live_resume_of_the_last_megabyte_from_hugging_face` seeds the partial with all but the last 1,048,576 bytes of the local model and resumes from Hugging Face: the range survives the 302 to the CDN, no progress report fell below the seeded size, and the result matches the pinned SHA-256 (46 s, most of it copying and hashing 1.6 GB).
- Found on the way: ureq 3's body `limit(n)` rejects a body of exactly n bytes, so the limit is the size plus one.
- Window check: release build, model moved aside, partial seeded with all but the last 100 MB; Felipe pressed Descargar modelo and saw "Modelo de voz listo." The log shows `downloading ... at byte 1519697675` then `model ready`; the file is 1,624,555,275 bytes. A full 1.6 GB download from nothing is not yet timed.

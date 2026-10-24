pub const CONFIRM_SUCCESS_HTML: &str = r#"<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Gilvave — Почта подтверждена</title>
    <link rel="icon" type="image/svg+xml" href="data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCA1MTIgNTEyIiB3aWR0aD0iNTEyIiBoZWlnaHQ9IjUxMiI+CiAgPGRlZnM+CiAgICA8cmFkaWFsR3JhZGllbnQgaWQ9ImktYmciIGN4PSI1MCUiIGN5PSIzOCUiIHI9Ijc1JSI+CiAgICAgIDxzdG9wIG9mZnNldD0iMCIgc3RvcC1jb2xvcj0iIzE0MWEyZSIvPgogICAgICA8c3RvcCBvZmZzZXQ9Ii42IiBzdG9wLWNvbG9yPSIjMGEwZTFhIi8+CiAgICAgIDxzdG9wIG9mZnNldD0iMSIgc3RvcC1jb2xvcj0iIzA0MDYwYyIvPgogICAgPC9yYWRpYWxHcmFkaWVudD4KICAgIDxsaW5lYXJHcmFkaWVudCBpZD0iaS13YXZlIiBncmFkaWVudFVuaXRzPSJ1c2VyU3BhY2VPblVzZSIgeDE9Ii0xNTAiIHkxPSItMTUwIiB4Mj0iMTUwIiB5Mj0iMTUwIj4KICAgICAgPHN0b3Agb2Zmc2V0PSIwIiBzdG9wLWNvbG9yPSIjM0ZFOUZGIi8+CiAgICAgIDxzdG9wIG9mZnNldD0iLjQiIHN0b3AtY29sb3I9IiM1RTdCRkYiLz4KICAgIDxzdG9wIG9mZnNldD0iLjciIHN0b3AtY29sb3I9IiM5QTVDRjYiLz4KICAgIDxzdG9wIG9mZnNldD0iMSIgc3RvcC1jb2xvcj0iI0ZGNUZBOCIvPgogICAgPC9saW5lYXJHcmFkaWVudD4KICAgIDxsaW5lYXJHcmFkaWVudCBpZD0iaS13YXJtIiBncmFkaWVudFVuaXRzPSJ1c2VyU3BhY2VPblVzZSIgeDE9Ii03MiIgeTE9IjAiIHgyPSIxNDgiIHkyPSIwIj4KICAgICAgPHN0b3Agb2Zmc2V0PSIwIiBzdG9wLWNvbG9yPSIjRkY0RjhGIi8+CiAgICAgIDxzdG9wIG9mZnNldD0iLjU1IiBzdG9wLWNvbG9yPSIjRkY3QTREIi8+CiAgICAgIDxzdG9wIG9mZnNldD0iMSIgc3RvcC1jb2xvcj0iI0ZGQzI0RiIvPgogICAgPC9saW5lYXJHcmFkaWVudD4KICAgIDxmaWx0ZXIgaWQ9ImktZ2xvdyIgeD0iLTQwJSIgeT0iLTQwJSIgd2lkdGg9IjE4MCUiIGhlaWdodD0iMTgwJSI+CiAgICAgIDxmZUdhdXNzaWFuQmx1ciBzdGREZXZpYXRpb249IjEyIiByZXN1bHQ9ImIiLz4KICAgICAgPGZlTWVyZ2U+PGZlTWVyZ2VOb2RlIGluPSJiIi8+PGZlTWVyZ2VOb2RlIGluPSJTb3VyY2VHcmFwaGljIi8+PC9mZU1lcmdlPgogICAgPC9maWx0ZXI+CiAgICA8ZmlsdGVyIGlkPSJpLXNvZnQiIHg9Ii0xMDAlIiB5PSItMTAwJSIgd2lkdGg9IjMwMCUiIGhlaWdodD0iMzAwJSI+CiAgICAgIDxmZUdhdXNzaWFuQmx1ciBzdGREZXZpYXRpb249IjM2Ii8+CiAgICA8L2ZpbHRlcj4KICAgIDxmaWx0ZXIgaWQ9Imktc3BhcmsiIHg9Ii0yMDAlIiB5PSItMjAwJSIgd2lkdGg9IjUwMCUiIGhlaWdodD0iNTAwJSI+CiAgICAgIDxmZUdhdXNzaWFuQmx1ciBzdGREZXZpYXRpb249IjgiLz4KICAgIDwvZmlsdGVyPgogIDwvZGVmcz4KCiAgPHJlY3Qgd2lkdGg9IjUxMiIgaGVpZ2h0PSI1MTIiIHJ4PSIxMTYiIGZpbGw9InVybCgjaS1iZykiLz4KCiAgPGcgZmlsbD0iI2NmZTJmZiI+CiAgICA8Y2lyY2xlIGN4PSIxMjgiIGN5PSIxMDYiIHI9IjIiIG9wYWNpdHk9Ii4zNSIvPgogICAgPGNpcmNsZSBjeD0iMjEyIiBjeT0iNzIiIHI9IjEuNiIgb3BhY2l0eT0iLjMiLz4KICAgIDxjaXJjbGUgY3g9IjMzMCIgY3k9IjkwIiByPSIyIiBvcGFjaXR5PSIuNCIvPgogICAgPGNpcmNsZSBjeD0iNDEwIiBjeT0iMTQwIiByPSIxLjciIG9wYWNpdHk9Ii4zIi8+CiAgPC9nPgoKICA8ZyB0cmFuc2Zvcm09InRyYW5zbGF0ZSgyNTYsMjU0KSBzY2FsZSgwLjg4KSIgZmlsbD0ibm9uZSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIj4KICAgIDxjaXJjbGUgY3g9IjAiIGN5PSIxMCIgcj0iMTUwIiBmaWxsPSIjM0I4MkY2IiBvcGFjaXR5PSIuMTQiIGZpbHRlcj0idXJsKCNpLXNvZnQpIi8+CiAgICA8Y2lyY2xlIGN4PSI0MCIgY3k9IjMwIiByPSIxMTAiIGZpbGw9IiNFQzQ4OTkiIG9wYWNpdHk9Ii4xMiIgZmlsdGVyPSJ1cmwoI2ktc29mdCkiLz4KCiAgICA8cGF0aCBkPSJNIC0xOCwxMDMgQSAxMDUgMTA1IDAgMCAxIC0xMDMsMTgiIHN0cm9rZT0iIzVFN0JGRiIgc3Ryb2tlLXdpZHRoPSIxMiIgb3BhY2l0eT0iLjM4Ii8+CgogICAgPHBhdGggZD0iTSA4NiwtMTIzIEEgMTUwIDE1MCAwIDEgMCAxNDgsMjQiIHN0cm9rZT0idXJsKCNpLXdhdmUpIiBzdHJva2Utd2lkdGg9IjMwIiBmaWx0ZXI9InVybCgjaS1nbG93KSIvPgoKICAgIDxwYXRoIGQ9Ik0gMTQ4LDI0IEwgOTgsMjQgTCA1MCwtNTkgTCAxMCwtMTEgTCAtMTgsLTQxIEwgLTU2LDI0IEwgLTcyLDI0IgogICAgICAgICAgc3Ryb2tlPSJ1cmwoI2ktd2FybSkiIHN0cm9rZS13aWR0aD0iMjYiLz4KCiAgICA8Y2lyY2xlIGN4PSI1MCIgY3k9Ii01OSIgcj0iMjYiIGZpbGw9IiNGRkI2NUEiIG9wYWNpdHk9Ii41NSIgZmlsdGVyPSJ1cmwoI2ktc3BhcmspIi8+CiAgICA8Y2lyY2xlIGN4PSI1MCIgY3k9Ii01OSIgcj0iNiIgZmlsbD0iI0ZGRjdFOCIvPgogICAgPGNpcmNsZSBjeD0iODYiIGN5PSItMTIzIiByPSIxNiIgZmlsbD0iIzM1RTZGRiIgb3BhY2l0eT0iLjUiIGZpbHRlcj0idXJsKCNpLXNwYXJrKSIvPgogICAgPGNpcmNsZSBjeD0iODYiIGN5PSItMTIzIiByPSI0LjUiIGZpbGw9IiNFQUZERkYiLz4KICA8L2c+Cjwvc3ZnPg==">
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: 'Segoe UI', -apple-system, BlinkMacSystemFont, Roboto, Helvetica, Arial, sans-serif;
            background-color: #0b0e18;
            background-image: 
                radial-gradient(circle at 50% 0%, #161b33 0%, transparent 65%),
                radial-gradient(circle at 100% 100%, #121026 0%, transparent 50%);
            background-attachment: fixed;
            color: #f1f5f9;
            display: flex;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
            padding: 20px;
        }
        .card {
            background-color: #15192b;
            border: 1px solid #2a3354;
            border-radius: 20px;
            padding: 44px 32px 36px;
            max-width: 440px;
            width: 100%;
            text-align: center;
            box-shadow: 0 24px 60px -12px rgba(0, 0, 0, 0.65), 0 0 1px 1px rgba(255, 255, 255, 0.05);
            position: relative;
            overflow: hidden;
        }
        .card::before {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            height: 3px;
            background: linear-gradient(90deg, #38bdf8 0%, #a855f7 50%, #ff4d79 100%);
        }
        .icon-container {
            position: relative;
            width: 72px;
            height: 72px;
            margin: 0 auto 24px;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .icon-halo {
            position: absolute;
            inset: -8px;
            border-radius: 9999px;
            background: radial-gradient(circle, rgba(45, 212, 191, 0.22) 0%, transparent 70%);
            animation: pulse 2.8s infinite ease-in-out;
        }
        .icon-bubble {
            width: 64px;
            height: 64px;
            border-radius: 9999px;
            background: rgba(45, 212, 191, 0.12);
            border: 1.5px solid rgba(45, 212, 191, 0.4);
            color: #2dd4bf;
            display: flex;
            align-items: center;
            justify-content: center;
            box-shadow: 0 0 25px rgba(45, 212, 191, 0.25);
            position: relative;
            z-index: 1;
        }
        .icon-bubble svg {
            width: 32px;
            height: 32px;
        }
        @keyframes pulse {
            0%, 100% { transform: scale(0.95); opacity: 0.6; }
            50% { transform: scale(1.1); opacity: 1; }
        }
        h1 {
            font-size: 24px;
            font-weight: 700;
            margin-bottom: 12px;
            color: #f1f5f9;
            letter-spacing: -0.02em;
        }
        p {
            font-size: 15px;
            color: #94a3b8;
            line-height: 1.6;
            margin-bottom: 28px;
        }
        p strong {
            color: #ffffff;
            font-weight: 600;
        }
        .btn {
            display: inline-flex;
            align-items: center;
            justify-content: center;
            width: 100%;
            background: linear-gradient(135deg, #5b6cf9 0%, #a855f7 48%, #ff4d79 100%);
            color: #ffffff;
            border: none;
            border-radius: 12px;
            padding: 13px 24px;
            font-size: 15px;
            font-weight: 600;
            letter-spacing: 0.01em;
            cursor: pointer;
            box-shadow: 0 4px 18px rgba(168, 85, 247, 0.35);
            transition: transform 0.15s ease, box-shadow 0.15s ease, filter 0.15s ease;
        }
        .btn:hover {
            filter: brightness(1.08);
            box-shadow: 0 6px 24px rgba(255, 77, 121, 0.45);
            transform: translateY(-1px);
        }
        .btn:active {
            transform: translateY(0);
        }
        .hint {
            margin-top: 18px;
            font-size: 13px;
            color: #64748b;
        }
    </style>
</head>
<body>
    <div class="card">
        <div class="icon-container">
            <div class="icon-halo"></div>
            <div class="icon-bubble">
                <svg fill="none" viewBox="0 0 24 24" stroke-width="2.5" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" d="m4.5 12.75 6 6 9-13.5" />
                </svg>
            </div>
        </div>
        <h1>Почта подтверждена!</h1>
        <p>Ваш аккаунт активирован. Теперь вы можете закрыть эту страницу и вернуться в приложение <strong>Gilvave</strong>.</p>
        <button class="btn" onclick="window.close()">Закрыть вкладку</button>
        <div class="hint">Если вкладка не закрылась автоматически, закройте её вручную.</div>
    </div>
</body>
</html>"#;

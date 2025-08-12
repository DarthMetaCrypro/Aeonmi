@echo off
set PYO3_PYTHON=%CD%\.venv\Scripts\python.exe
set PYTHONHOME=%CD%\.venv
set PATH=%CD%\.venv\Scripts;%CD%\.venv;%PATH%
echo Python = %PYO3_PYTHON%

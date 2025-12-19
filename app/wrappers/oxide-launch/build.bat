@echo off
REM Build script for OxideLaunch.jar
REM This creates the wrapper JAR without requiring Gradle

setlocal enabledelayedexpansion

set "SRC_DIR=src\main\java"
set "OUT_DIR=build\classes"
set "JAR_NAME=OxideLaunch.jar"

echo Building OxideLaunch...

REM Clean previous build
if exist "%OUT_DIR%" rmdir /s /q "%OUT_DIR%"

REM Create output directory
mkdir "%OUT_DIR%"

REM Find all Java files
set "JAVA_FILES="
for /r "%SRC_DIR%" %%f in (*.java) do (
    set "JAVA_FILES=!JAVA_FILES! "%%f""
)

REM Compile with Java 8 target and warnings enabled
echo Compiling Java sources...
javac -d "%OUT_DIR%" -source 8 -target 8 -encoding UTF-8 -Xlint:all,-serial %JAVA_FILES%

if %ERRORLEVEL% neq 0 (
    echo Compilation failed!
    exit /b 1
)

REM Create manifest
echo Main-Class: dev.oxide.launch.OxideLaunch> "%OUT_DIR%\MANIFEST.MF"
echo.>> "%OUT_DIR%\MANIFEST.MF"

REM Create JAR
echo Creating JAR...
pushd "%OUT_DIR%"
jar cfm "..\%JAR_NAME%" MANIFEST.MF dev\*
popd

if exist "build\%JAR_NAME%" (
    echo.
    echo Build successful: build\%JAR_NAME%
    dir /b "build\%JAR_NAME%"
) else (
    echo Build failed - JAR not created!
    exit /b 1
)

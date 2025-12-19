#!/bin/bash
# Build script for OxideLaunch.jar
# This creates the wrapper JAR without requiring Gradle

set -e

SRC_DIR="src/main/java"
OUT_DIR="build/classes"
JAR_NAME="OxideLaunch.jar"

echo "Building OxideLaunch..."

# Clean previous build
rm -rf "$OUT_DIR"

# Create output directory
mkdir -p "$OUT_DIR"

# Find all Java files
JAVA_FILES=$(find "$SRC_DIR" -name "*.java")

# Compile with Java 8 target and warnings enabled
echo "Compiling Java sources..."
javac -d "$OUT_DIR" -source 8 -target 8 -encoding UTF-8 -Xlint:all,-serial $JAVA_FILES

if [ $? -ne 0 ]; then
    echo "Compilation failed!"
    exit 1
fi

# Create manifest
echo "Main-Class: dev.oxide.launch.OxideLaunch" > "$OUT_DIR/MANIFEST.MF"
echo "" >> "$OUT_DIR/MANIFEST.MF"

# Create JAR
echo "Creating JAR..."
pushd "$OUT_DIR" > /dev/null
jar cfm "../$JAR_NAME" MANIFEST.MF dev/
popd > /dev/null

if [ -f "build/$JAR_NAME" ]; then
    echo ""
    echo "Build successful: build/$JAR_NAME"
    ls -la "build/$JAR_NAME"
else
    echo "Build failed - JAR not created!"
    exit 1
fi

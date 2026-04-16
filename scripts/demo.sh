#!/bin/bash
# Terminal demo for README / Show HN

BECK="./target/release/beck"

echo "$ $BECK --help"
script -q /dev/null $BECK --help 2>&1 | head -15
echo ""
echo "---"
echo ""

echo "$ $BECK sync"
$BECK sync 2>&1
echo ""
echo "---"
echo ""

echo "$ $BECK bench"
$BECK bench 2>&1
echo ""
echo "---"
echo ""

echo "$ $BECK query \"transcribe audio\""
$BECK query "transcribe audio" 2>&1
echo ""
echo "---"
echo ""

echo "$ $BECK load whisper"
$BECK load whisper 2>&1 | head -10
echo ""
echo "---"
echo ""

echo "$ $BECK prompt"
$BECK prompt 2>&1

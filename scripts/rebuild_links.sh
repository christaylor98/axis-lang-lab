#!/usr/bin/env sh
set -e

echo "NOTE: Link table management removed from bridge"
echo ""
echo "External libraries are now linked directly via CLI flags:"
echo "  --link-lib <library-file>"
echo "  --link-search <directory>"
echo ""
echo "Example usage:"
echo "  ./scripts/build_ai1.sh input.ax output --link-lib libaxis_std.a --link-search ./libs"
echo ""
echo "This script is deprecated and will be removed."

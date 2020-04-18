#!/bin/bash
FLAMEGRAPH=../../FlameGraph
$FLAMEGRAPH/stackcollapse.pl out.stacks | $FLAMEGRAPH/flamegraph.pl > graph.svg
echo "Written SVG image to graph.svg"

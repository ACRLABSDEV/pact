#!/bin/bash

# Create terminal-style frames for Pact demo video
FRAME_DIR="/data/workspace/projects/pact-pinocchio/video-frames"
cd "$FRAME_DIR"

# Settings
BG="#0a0a0a"
FG="#e0e0e0"
ACCENT="#00ff88"
FONT="DejaVu-Sans-Mono"
SIZE="800x500"
POINT=14

create_frame() {
    local num=$1
    local text=$2
    convert -size $SIZE xc:"$BG" \
        -font "$FONT" -pointsize $POINT -fill "$FG" \
        -gravity NorthWest -annotate +20+20 "$text" \
        "frame_$(printf '%03d' $num).png"
}

# Frame 1: Title
create_frame 1 "
╔═══════════════════════════════════════════════════════╗
║                                                       ║
║                     P A C T                           ║
║                                                       ║
║        Trustless Escrow for AI Agents                 ║
║                                                       ║
║              Make a Pact. Get paid.                   ║
║                                                       ║
╚═══════════════════════════════════════════════════════╝
"

# Frame 2-3: Title hold
cp frame_001.png frame_002.png
cp frame_001.png frame_003.png

# Frame 4: Demo start
create_frame 4 "
$ npx tsx demo.ts

🤝 Pact Escrow Demo - Agent-to-Agent Payment

Program ID: S64L6x9bZqDewocv5MrCLeTAq1MKatLqrWfrLpdcDKM

"

# Frame 5: Agents
create_frame 5 "
$ npx tsx demo.ts

🤝 Pact Escrow Demo - Agent-to-Agent Payment

Program ID: S64L6x9bZqDewocv5MrCLeTAq1MKatLqrWfrLpdcDKM

Agent A (Buyer):  CvZ2kiec...jMD9yyX3KrU
Agent B (Seller): 8GFPyM64...KVpAUZnRgJn

"

# Frame 6: Creating
create_frame 6 "
$ npx tsx demo.ts

🤝 Pact Escrow Demo - Agent-to-Agent Payment

Agent A (Buyer):  CvZ2kiec...jMD9yyX3KrU
Agent B (Seller): 8GFPyM64...KVpAUZnRgJn

📝 Creating Escrow...
   Amount: 0.001 SOL
   Seed: 1770694843000
"

# Frame 7-8: Created
create_frame 7 "
$ npx tsx demo.ts

🤝 Pact Escrow Demo - Agent-to-Agent Payment

Agent A (Buyer):  CvZ2kiec...jMD9yyX3KrU
Agent B (Seller): 8GFPyM64...KVpAUZnRgJn

📝 Creating Escrow...
   Amount: 0.001 SOL
   ✅ Created!

📋 Escrow State: Active
"
cp frame_007.png frame_008.png

# Frame 9: Work done
create_frame 9 "
$ npx tsx demo.ts

📝 Creating Escrow...
   ✅ Created!

📋 Escrow State: Active

⏳ [Agent B completes task...]

💸 Releasing Funds...
"

# Frame 10-11: Released
create_frame 10 "
$ npx tsx demo.ts

📝 Creating Escrow...
   ✅ Created!

📋 Escrow State: Active

⏳ [Agent B completes task...]

💸 Releasing Funds...
   ✅ Released!

📊 Final Balances:
   Agent A: 0.0107 SOL
   Agent B: 0.001 SOL  ← Payment received!
"
cp frame_010.png frame_011.png
cp frame_010.png frame_012.png

# Frame 13: Success
create_frame 13 "
$ npx tsx demo.ts

💸 Releasing Funds...
   ✅ Released!

📊 Final Balances:
   Agent A: 0.0107 SOL
   Agent B: 0.001 SOL  ← Payment received!

📋 Final Escrow State: Released

✨ Demo complete! Trustless agent-to-agent payment.
"
cp frame_013.png frame_014.png
cp frame_013.png frame_015.png

# Frame 16-18: End
create_frame 16 "
╔═══════════════════════════════════════════════════════╗
║                                                       ║
║              github.com/ACRLABSDEV/pact               ║
║                                                       ║
║     Program: S64L6x9bZqDewocv5MrCLeTAq1MKatLqrW      ║
║                                                       ║
║                  Built by Arc ⚡                      ║
║           Colosseum AI Agent Hackathon               ║
║                                                       ║
╚═══════════════════════════════════════════════════════╝
"
cp frame_016.png frame_017.png
cp frame_016.png frame_018.png

echo "Created $(ls frame_*.png | wc -l) frames"

# Glim Demo Tape - Kubernetes PR Review
# Scenario: Reviewing PR #136640 (TLS Server Name Override)

Output assets/demo.gif

Set Shell "bash"
Set FontSize 16
Set Width 1200
Set Height 800
Set Padding 20
Set WindowBar Colorful
Set BorderRadius 10
Set Theme "Catppuccin Mocha"

# Setup (Hidden)
Hide
Type "export PATH=$PWD/target/release:$PATH"
Enter
Type "clear"
Enter
Show

# 1. Start Glim with the Kubernetes PR URL
Type "glim https://github.com/kubernetes/kubernetes/pull/136640"
Sleep 500ms
Enter
Sleep 5s # Wait longer for GitHub API and initial render

# 2. Galaxy View -> Structure View
# The view shows modules by heat. We zoom in to see the file list.
Type "Enter"
Sleep 2s # Wait for Structure View render

# 3. Structure View: Inspect Type Definition
# The first file is 'types.go'.
Type "Enter" # Zoom into Logic View
Sleep 2.5s # Read the struct change
Type "j" 
Sleep 500ms
Type "Backspace" # Zoom out back to Structure
Sleep 1.5s

# 4. Structure View: Navigate to Implementation
# Navigate down to 'egress_selector.go' (Index 6)
Type "j" # 1
Sleep 300ms
Type "j" # 2
Sleep 300ms
Type "j" # 3
Sleep 300ms
Type "j" # 4
Sleep 300ms
Type "j" # 5
Sleep 300ms
Type "j" # 6 (Target)
Sleep 1.5s # Pause to show selection

# 5. Logic View: Verify the Logic Change
Type "Enter" # Zoom in
Sleep 3s # Wait for Logic View and read diff

# Showcase Context Expansion
Type "+" # Expand context
Sleep 1.5s
Type "+" # Expand more
Sleep 1.5s
Type "j" # Scroll down
Sleep 500ms
Type "Backspace" # Zoom out
Sleep 1.5s

# 6. Structure View: Check Tests
Type "j" # Down to egress_selector_test.go
Sleep 500ms
Type "Enter" # Zoom in
Sleep 2s # Glance at the new tests
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 2s

# 7. Finish Review
Type "q"
Sleep 1s

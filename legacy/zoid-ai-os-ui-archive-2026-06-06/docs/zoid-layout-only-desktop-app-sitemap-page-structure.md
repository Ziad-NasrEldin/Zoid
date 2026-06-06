Below is a layout-only desktop app sitemap and page structure for Zoid, based on the uploaded product requirements file. I am excluding visual styling, colors, typography, motion, and theme direction as requested.

1. Product Sitemap
Zoid Desktop App
│
├── Today
│   ├── Today Dashboard
│   ├── Daily Brief Detail
│   ├── Needs Attention
│   ├── Active Work
│   ├── Today Tasks
│   ├── Calendar Preview
│   ├── Content Queue Preview
│   ├── Active Agents Preview
│   ├── Repo Attention Preview
│   ├── Automation Failures Preview
│   ├── Gmail Summary Preview
│   └── Recent History
│
├── Code
│   ├── Code Dashboard
│   ├── Repo Discovery
│   ├── Managed Repositories
│   │   └── Repository Detail
│   │       ├── Overview
│   │       ├── Git Status
│   │       ├── Changed Files
│   │       ├── Diff Viewer
│   │       ├── Branches
│   │       ├── Commits
│   │       ├── Pull Requests
│   │       ├── GitHub Checks
│   │       ├── Vercel Deployments
│   │       ├── Launch Gate
│   │       ├── Linked Tasks
│   │       ├── Linked Agent Runs
│   │       ├── Linked Notes
│   │       ├── Linked Product
│   │       └── Repo History
│   ├── GitHub
│   │   ├── PRs
│   │   ├── Branches
│   │   ├── Commits
│   │   ├── Checks
│   │   └── Issues
│   ├── Vercel
│   │   ├── Projects
│   │   ├── Deployments
│   │   ├── Preview URLs
│   │   ├── Production URLs
│   │   └── Build Logs
│   ├── Launch Gates
│   ├── Code Review Records
│   └── Code Settings
│
├── Agents
│   ├── Agents Dashboard
│   ├── Active Runs
│   ├── Agent Run History
│   │   └── Agent Run Detail
│   │       ├── Summary
│   │       ├── Prompt
│   │       ├── Clean Output
│   │       ├── Raw Logs
│   │       ├── Status Timeline
│   │       ├── Linked Task
│   │       ├── Linked Repo/Product/Content
│   │       ├── Review Record
│   │       └── Events
│   ├── Agent Profiles
│   │   └── Agent Profile Detail/Edit
│   ├── Reviewer Agent
│   ├── Session Console
│   ├── CLI Profiles
│   └── Agent Settings
│
├── Notes
│   ├── Notes Dashboard
│   ├── All Notes
│   │   └── Note Detail / Editor
│   ├── Collections
│   │   └── Collection Detail
│   ├── Tags
│   │   └── Tag Detail
│   ├── Daily Notes
│   ├── Code Notes
│   ├── Content Notes
│   ├── Business Notes
│   ├── Product Notes
│   ├── Imported Apple Notes
│   ├── Backlinks
│   ├── Entity-Linked Notes
│   └── Notes Settings
│
├── Content
│   ├── Content Dashboard
│   ├── Content Calendar
│   ├── Content Plans
│   │   └── Content Plan Detail
│   ├── Content Pieces
│   │   └── Content Piece Detail
│   │       ├── Brief
│   │       ├── Draft
│   │       ├── Assets
│   │       ├── Captions
│   │       ├── Platform Adaptations
│   │       ├── Review
│   │       ├── Publishing
│   │       ├── Verification
│   │       └── History
│   ├── AI Intel Brief Pipeline
│   ├── Enterprise Carousel Pipeline
│   ├── Asset Library
│   ├── Review Queue
│   ├── OmniSocials
│   │   ├── Account Status
│   │   ├── Media Uploads
│   │   ├── Scheduled Posts
│   │   ├── Published Posts
│   │   └── Failed Posts
│   ├── Publishing History
│   ├── Failure Reports
│   └── Content Settings
│
├── Automations
│   ├── Automations Dashboard
│   ├── All Automations
│   │   └── Automation Detail
│   │       ├── Overview
│   │       ├── Schedule
│   │       ├── Runs
│   │       ├── Logs
│   │       ├── Linked Entities
│   │       ├── Failure Policy
│   │       └── History
│   ├── Automation Runs
│   │   └── Automation Run Detail
│   ├── Failed Runs
│   ├── Schedules
│   ├── Default Automations
│   │   ├── Daily MaVoid Content Publishing
│   │   ├── Morning AI Brief
│   │   ├── Repo Scanner
│   │   ├── Agent Monitor
│   │   ├── Notes Indexer
│   │   ├── Content Calendar Monitor
│   │   └── Deployment Verifier
│   └── Automation Settings
│
├── Business
│   ├── Business Dashboard
│   ├── Contacts
│   │   └── Contact Detail
│   │       ├── Overview
│   │       ├── Emails
│   │       ├── Tasks
│   │       ├── Notes
│   │       ├── Products
│   │       ├── Follow-ups
│   │       └── History
│   ├── Companies / Clients
│   │   └── Company Detail
│   │       ├── Overview
│   │       ├── Contacts
│   │       ├── Emails
│   │       ├── Tasks
│   │       ├── Notes
│   │       ├── Products
│   │       ├── Follow-ups
│   │       └── History
│   ├── Follow-ups
│   ├── Linked Gmail
│   ├── Business Tasks
│   ├── SOP Notes
│   └── Business Settings
│
├── Products
│   ├── Products Dashboard
│   ├── All Products
│   │   └── Product Detail
│   │       ├── Overview
│   │       ├── Roadmap
│   │       ├── Tasks
│   │       ├── Repos
│   │       ├── Deployments
│   │       ├── Releases
│   │       ├── Notes / Decisions
│   │       ├── Content
│   │       ├── Automations
│   │       ├── Clients / Companies
│   │       └── Timeline / History
│   ├── Roadmap Items
│   ├── Product Launches
│   ├── Product Decisions
│   └── Product Settings
│
├── Files
│   ├── Files Dashboard
│   ├── File Explorer
│   │   └── File / Folder Detail
│   ├── Pinned Folders
│   ├── Zoid Folder
│   │   ├── Notes
│   │   ├── Content
│   │   ├── Assets
│   │   ├── Exports
│   │   ├── Imports
│   │   ├── Files
│   │   └── Products
│   ├── Repositories
│   ├── Content Assets
│   ├── Browser Captures
│   ├── Imports
│   ├── Exports
│   ├── Linked Files
│   └── File Settings
│
├── Browser
│   ├── Browser Workspace
│   ├── Active Tabs
│   ├── Pinned Tabs
│   ├── Work Bookmarks
│   ├── Work History
│   ├── Saved Pages
│   ├── Screenshots / Captures
│   ├── App Verification Sessions
│   └── Browser Settings
│
├── Inbox
│   ├── Inbox Dashboard
│   ├── All Attention Items
│   ├── Zoid Notifications
│   ├── Agent Completions
│   ├── Agent Blockers
│   ├── Automation Failures
│   ├── Review Approvals
│   ├── Required Fixes
│   ├── Content Publishing Status
│   ├── Calendar / Task Reminders
│   ├── Gmail
│   │   ├── Recent Messages
│   │   ├── Search Messages
│   │   ├── Thread Detail
│   │   ├── Draft Reply
│   │   └── Compose Email
│   └── Inbox Settings
│
├── Shared System Pages
│   ├── Tasks
│   │   ├── Task Inbox
│   │   ├── Todo
│   │   ├── In Progress
│   │   ├── Waiting / Blocked
│   │   ├── Needs Review
│   │   ├── Done
│   │   ├── Archived
│   │   └── Task Detail
│   ├── Calendar
│   │   ├── Today
│   │   ├── Week
│   │   ├── Month
│   │   ├── Content Calendar
│   │   ├── Automation Schedule
│   │   └── Workspace Timeline
│   ├── History
│   │   ├── Global Recent History
│   │   ├── Today Activity
│   │   ├── Workspace History
│   │   └── Entity History
│   ├── Reviews
│   │   ├── Pending Reviews
│   │   ├── Approved
│   │   ├── Required Fixes
│   │   ├── Blocked
│   │   └── Review Record Detail
│   ├── Permissions
│   ├── Integrations
│   ├── Notifications
│   └── Local Profile
│
└── Settings
    ├── General
    ├── Workspaces
    ├── Widgets
    ├── Storage
    ├── CLI / Agent Profiles
    ├── Review Gate
    ├── Permissions
    ├── Integrations
    │   ├── Gmail
    │   ├── Apple Calendar
    │   ├── GitHub
    │   ├── Vercel
    │   └── OmniSocials
    ├── Notifications
    ├── Security
    ├── Data / Indexing
    ├── Import / Migration
    ├── Export
    └── About
2. Global Desktop App Layout

Every page should sit inside the same desktop shell.

2.1 App Shell Structure
Desktop Window
│
├── Left Sidebar
│   ├── Workspace Navigation
│   ├── Pinned Objects / Current Focus
│   └── Bottom Utilities
│
├── Main Workspace Area
│   ├── Workspace Header
│   ├── Optional Secondary Navigation
│   ├── Main Page Body
│   └── Optional Local Footer / Status Strip
│
├── Right Inspector Panel
│   └── Selected Object Details
│
├── Sessions Panel
│   ├── Clean Conversation Mode
│   └── Raw Terminal Mode
│
└── Global Overlays
    ├── Command Palette
    ├── Search Panel
    ├── Notification Center
    ├── Permission Prompt
    ├── Review Gate Prompt
    ├── Confirmation Dialog
    ├── Entity Link Picker
    └── File Picker
2.2 Left Sidebar
Order
Left Sidebar
│
├── App / Workspace Switch Area
│
├── Primary Workspaces
│   ├── Today
│   ├── Code
│   ├── Agents
│   ├── Notes
│   ├── Content
│   ├── Automations
│   ├── Business
│   ├── Products
│   ├── Files
│   ├── Browser
│   └── Inbox
│
├── Optional Current Focus Section
│   ├── Active Task
│   ├── Active Agent Run
│   ├── Current Product
│   └── Current Repo
│
└── Utility Section
    ├── Tasks
    ├── Calendar
    ├── History
    ├── Notifications
    ├── Settings
    └── Local Profile
Sidebar buttons
Button	Location	Action
Workspace item	Primary workspace list	Opens workspace dashboard
Current focus item	Current Focus section	Opens related task, run, repo, or product
Tasks	Utility section	Opens global task view
Calendar	Utility section	Opens calendar
History	Utility section	Opens global event history
Notifications	Utility section	Opens notification center
Settings	Utility section	Opens settings
Local Profile	Utility section	Opens local profile page
2.3 Workspace Header

Every workspace uses a consistent header pattern.

Workspace Header
│
├── Left Area
│   ├── Workspace Title
│   ├── Optional Context Breadcrumb
│   └── Optional Status Summary
│
├── Center Area
│   ├── Workspace Search
│   ├── Scope Selector
│   └── Filter Chips
│
└── Right Area
    ├── Primary Action Button
    ├── Secondary Action Button(s)
    ├── View Toggle
    ├── Refresh / Sync Button
    └── More Menu
Common header buttons
Button	Use
New	Creates the primary object for the workspace
Add	Adds an external object, folder, repo, profile, or integration
Run	Starts an agent, automation, check, verifier, or command
Review	Opens or requests review gate
Search	Searches within the current workspace
Filter	Opens filter controls
Sort	Opens sorting controls
View	Switches between list, board, calendar, timeline, or detail views
Refresh	Reloads local and integration data
More	Opens secondary actions
2.4 Main Page Body Patterns

Zoid should reuse several structural patterns.

Dashboard page pattern
Dashboard Body
│
├── Priority Row
│   ├── Main Summary Card
│   ├── Needs Attention Card
│   └── Active Status Card
│
├── Primary Work Area
│   ├── Widget / Panel 1
│   ├── Widget / Panel 2
│   ├── Widget / Panel 3
│   └── Widget / Panel 4
│
├── Secondary Work Area
│   ├── Queue / List Panel
│   ├── Timeline / History Panel
│   └── Linked Items Panel
│
└── Optional Bottom Section
    ├── Recent Events
    └── Suggested Next Actions
List page pattern
List Page Body
│
├── List Toolbar
│   ├── Search
│   ├── Filters
│   ├── Sort
│   ├── Group By
│   └── Bulk Actions
│
├── Item List / Table / Board
│   ├── Row / Card
│   ├── Row / Card
│   └── Row / Card
│
└── Pagination / Load More / Status Footer
Detail page pattern
Detail Page Body
│
├── Detail Header
│   ├── Title
│   ├── Status
│   ├── Metadata
│   └── Primary Actions
│
├── Detail Tabs
│   ├── Overview
│   ├── Work Area
│   ├── Linked Items
│   ├── Reviews
│   └── History
│
├── Main Detail Content
│
└── Right Inspector
Editor page pattern
Editor Page
│
├── Editor Header
│   ├── Object Title
│   ├── Save Status
│   ├── Linked Entities
│   └── Actions
│
├── Editor Body
│   ├── Main Text / Content Area
│   └── Optional Side Metadata Panel
│
└── Bottom Status
    ├── Last Saved
    ├── File Path
    └── Indexing Status
2.5 Right Inspector Panel

The right inspector changes based on the selected object.

Inspector Panel
│
├── Selected Entity Header
│   ├── Entity Type
│   ├── Title
│   ├── Status
│   └── Quick Actions
│
├── Metadata Section
│   ├── Owner / Workspace
│   ├── Created / Updated
│   ├── Source
│   ├── Priority / Attention Level
│   └── Tags
│
├── Linked Entities Section
│   ├── Tasks
│   ├── Notes
│   ├── Files
│   ├── Repos
│   ├── Products
│   ├── Content
│   ├── Contacts / Companies
│   └── URLs
│
├── Review / Permission Section
│   ├── Review Status
│   ├── Required Evidence
│   ├── Last Verdict
│   └── Pending Confirmation
│
├── Recent Activity
│
└── Inspector Actions
    ├── Open Full Page
    ├── Link Entity
    ├── Add Note
    ├── Create Task
    ├── Run Agent
    ├── View History
    └── More
2.6 Sessions Panel

Sessions are available globally but most commonly opened from Today, Code, Agents, Content, Automations, Browser, and Inbox.

Sessions Panel
│
├── Session Header
│   ├── Session Title
│   ├── Linked Entity
│   ├── Status
│   ├── Duration
│   └── Controls
│
├── Session Tabs
│   ├── Clean Mode
│   └── Raw Mode
│
├── Clean Mode Body
│   ├── User Prompt Bubble
│   ├── Command Card
│   ├── Progress Card
│   ├── Output Card
│   ├── Error Card
│   ├── Summary Card
│   └── Suggested Next Actions
│
├── Raw Mode Body
│   ├── Terminal Output
│   └── Input Line
│
└── Session Footer
    ├── Input Box
    ├── Attach Context Button
    ├── Send / Run Button
    ├── Stop Button
    ├── Retry Button
    ├── Request Review Button
    └── Open External Terminal Button
Sessions panel buttons
Button	Action
New Session	Starts a new session
Attach Context	Links task, repo, note, content piece, product, file, or URL
Run	Starts the CLI command or agent
Send	Sends prompt/input into session
Stop	Stops current run
Retry	Restarts failed or completed run with same context
Request Review	Starts reviewer flow
Clean Mode	Shows conversational cards
Raw Mode	Shows terminal output
Collapse Logs	Hides verbose logs
Open Logs	Opens full log view
Open External Terminal	Opens fallback terminal
Create Task	Creates task from session
Link to Entity	Links session to existing object
3. Global Action and Button System

This is not visual styling. It is functional button behavior.

3.1 Button Categories
Category	Examples	Confirmation needed?
Navigation	Open, View, Back, Breadcrumb	No
Creation	New Task, New Note, New Content Piece	Usually no
Drafting	Draft Reply, Generate Draft, Create Note	Usually no
Execution	Run Agent, Run Automation, Run Verification	Sometimes
Integration	Connect Gmail, Sync GitHub, Upload Media	Depends
Review	Request Review, Approve, Require Fixes	Yes for verdicts
Publishing	Schedule, Publish Now, Send Email	Always
Deployment	Deploy, Redeploy, Merge, Push	Always
File Operations	Move, Rename, Copy, Duplicate	Sometimes
Destructive	Delete, Disable, Remove, Bulk Delete	Always
3.2 Standard Confirmation Flow
User clicks consequential action
│
├── Permission Policy Check
│
├── If reviewer required:
│   ├── Open Review Gate Prompt
│   ├── Run reviewer agent
│   ├── Show ReviewRecord
│   └── Require user confirmation
│
├── If human confirmation required:
│   ├── Open Confirmation Dialog
│   ├── Show action summary
│   ├── Show affected entities
│   ├── Show irreversible/destructive warning if relevant
│   └── Confirm / Cancel
│
└── If approved:
    ├── Execute action
    ├── Record Event
    ├── Update entity status
    └── Notify if needed
3.3 Standard Confirmation Dialog Structure
Confirmation Dialog
│
├── Dialog Title
├── Action Summary
├── Affected Items List
├── Required Policy Explanation
├── Optional Review Evidence
├── Optional Checkbox for destructive actions
└── Footer Buttons
    ├── Cancel
    └── Confirm Action
3.4 Review Gate Prompt Structure
Review Gate Prompt
│
├── Reviewed Entity
├── Proposed Action
├── Evidence Required
├── Reviewer Agent Selector
├── Run Review Button
├── Review Result
│   ├── Verdict
│   ├── Evidence Summary
│   └── Required Fixes
└── Footer Buttons
    ├── Cancel
    ├── Apply Fixes / Re-run
    └── Approve and Continue
4. Today Workspace
4.1 Purpose

Today is the daily operating surface. It should gather current priorities, blockers, active agents, calendar, content queue, dirty repos, automation failures, Gmail summary, and recent history.

4.2 Sitemap
Today
├── Today Dashboard
├── Daily Brief Detail
├── Needs Attention
├── Active Work
├── Today Tasks
├── Calendar Preview
├── Content Queue Preview
├── Active Agents Preview
├── Repo Attention Preview
├── Automation Failures Preview
├── Gmail Summary Preview
└── Recent History
4.3 Today Dashboard Layout
Today Dashboard
│
├── Workspace Header
│   ├── Title: Today
│   ├── Date / Current Day Context
│   ├── Search Today
│   ├── New Task Button
│   ├── Start Agent Button
│   ├── Refresh Brief Button
│   └── Configure Widgets Button
│
├── Top Priority Row
│   ├── AI Daily Brief Widget
│   ├── Needs Attention Widget
│   └── Active Work Widget
│
├── Main Operations Grid
│   ├── Today Tasks Widget
│   ├── Calendar Widget
│   ├── Content Queue Widget
│   └── Active Agents Widget
│
├── Secondary Operations Grid
│   ├── Dirty / Attention Repos Widget
│   ├── Automation Failures Widget
│   ├── Gmail Summary Widget
│   └── Recent History Widget
│
└── Optional Right Inspector
    └── Details for selected task, alert, event, agent, repo, or content item
4.4 Today Header Buttons
Button	Action
New Task	Opens task creation dialog with workspace set to Today
Start Agent	Opens agent run dialog with optional task context
Refresh Brief	Rebuilds or reloads the daily brief
Configure Widgets	Opens widget edit mode
Search Today	Searches tasks, alerts, events, content, calendar, agents, Gmail summary, and repos shown in Today
4.5 AI Daily Brief Widget
AI Daily Brief Widget
│
├── Brief Summary
├── Priority Highlights
├── Known Blockers
├── Important Deadlines
├── Suggested Focus Order
└── Buttons
    ├── Open Full Brief
    ├── Refresh
    ├── Create Task from Brief Item
    └── Dismiss Item
4.6 Needs Attention Widget
Needs Attention Widget
│
├── Attention Item List
│   ├── Blocked Agent
│   ├── Failed Automation
│   ├── Failed Build/Test/Deploy
│   ├── Content Awaiting Review
│   ├── Publishing Failure
│   ├── Gmail Send Awaiting Confirmation
│   ├── Calendar / Task Due Soon
│   ├── Dirty Repo Alert
│   ├── Launch Gate Pending
│   └── Review Required / Required Fixes
└── Buttons per item
    ├── Open
    ├── Review
    ├── Resolve
    ├── Snooze
    └── Link to Task
4.7 Active Work Widget
Active Work Widget
│
├── Current Tasks
├── Current Products
├── Current Repos
├── Running Sessions
├── Active Agent Runs
└── Buttons
    ├── Continue
    ├── Open Session
    ├── Create Follow-up
    └── Mark Done
4.8 Today Tasks Widget
Today Tasks Widget
│
├── Task Filters
│   ├── Due Today
│   ├── Priority
│   ├── Blocked
│   ├── Needs Review
│   └── In Progress
├── Task List
└── Buttons
    ├── New Task
    ├── Open Task
    ├── Start
    ├── Mark Done
    ├── Block
    ├── Request Review
    └── Attach Agent Run
4.9 Calendar Widget
Calendar Widget
│
├── Upcoming Events
├── Task Due Items
├── Content Publishing Slots
├── Automation Schedule Items
├── Follow-ups
└── Buttons
    ├── Open Calendar
    ├── Create Event
    ├── Link Event to Task
    ├── Check Conflicts
    └── Open Source Event
4.10 Content Queue Widget
Content Queue Widget
│
├── Today’s Planned Posts
├── Drafts Needing Review
├── Assets Needing Completion
├── Scheduled Posts
├── Failed Posts
└── Buttons
    ├── Open Content Piece
    ├── Generate
    ├── Review
    ├── Upload Media
    ├── Schedule
    └── Verify
4.11 Active Agents Widget
Active Agents Widget
│
├── Running Agent Runs
├── Blocked Runs
├── Completed Runs Awaiting Review
├── Failed Runs
└── Buttons
    ├── Open Run
    ├── Open Session
    ├── Stop
    ├── Retry
    ├── Request Review
    └── Create Task
4.12 Dirty / Attention Repos Widget
Dirty / Attention Repos Widget
│
├── Repo List
│   ├── Repo Name
│   ├── Branch
│   ├── Changed Files Count
│   ├── Test/Build Status
│   ├── Launch Gate State
│   └── Last Event
└── Buttons
    ├── Open Repo
    ├── View Diff
    ├── Run Checks
    ├── Start Agent
    └── Open Launch Gate
4.13 Automation Failures Widget
Automation Failures Widget
│
├── Failed Automation Runs
├── Failure Summary
├── Last Run Time
├── Next Scheduled Run
└── Buttons
    ├── Open Run
    ├── View Logs
    ├── Retry
    ├── Pause Automation
    └── Create Fix Task
4.14 Gmail Summary Widget
Gmail Summary Widget
│
├── Recent Important Messages
├── Threads Needing Response
├── Drafts Awaiting Confirmation
├── Follow-up Candidates
└── Buttons
    ├── Open Gmail Inbox
    ├── Open Thread
    ├── Summarize
    ├── Draft Reply
    ├── Convert to Task
    └── Link to Contact
4.15 Recent History Widget
Recent History Widget
│
├── Event Stream
│   ├── Timestamp
│   ├── Actor
│   ├── Workspace
│   ├── Summary
│   ├── Severity
│   └── Linked Entity
└── Buttons
    ├── Open Event
    ├── Open Entity
    ├── Filter History
    └── View Full History
5. Code Workspace
5.1 Purpose

Code manages repos, Git state, GitHub, Vercel, review records, deployments, and Launch Gate.

5.2 Sitemap
Code
├── Code Dashboard
├── Repo Discovery
├── Managed Repositories
│   └── Repository Detail
│       ├── Overview
│       ├── Git Status
│       ├── Changed Files
│       ├── Diff Viewer
│       ├── Branches
│       ├── Commits
│       ├── Pull Requests
│       ├── GitHub Checks
│       ├── Vercel Deployments
│       ├── Launch Gate
│       ├── Linked Tasks
│       ├── Linked Agent Runs
│       ├── Linked Notes
│       ├── Linked Product
│       └── Repo History
├── GitHub
├── Vercel
├── Launch Gates
├── Code Review Records
└── Code Settings
5.3 Code Dashboard Layout
Code Dashboard
│
├── Workspace Header
│   ├── Title: Code
│   ├── Search Repos
│   ├── Add Repo Button
│   ├── Scan Folders Button
│   ├── Refresh Status Button
│   └── Code Settings Button
│
├── Summary Row
│   ├── Managed Repos Summary
│   ├── Dirty Repos Summary
│   ├── Open Launch Gates Summary
│   └── Failed Checks Summary
│
├── Main Body
│   ├── Managed Repositories List
│   ├── Repos Needing Attention
│   ├── Active Code Agent Runs
│   └── Recent Code Events
│
└── Right Inspector
    └── Selected repo, PR, deployment, Launch Gate, or task
5.4 Code Dashboard Buttons
Button	Action
Add Repo	Opens manual repo picker
Scan Folders	Opens repo discovery flow
Refresh Status	Reloads Git/GitHub/Vercel status
New Code Task	Creates a task linked to Code
Start Builder Agent	Starts agent with repo context
Open Launch Gates	Opens all Launch Gates
Code Settings	Opens repo/integration settings
5.5 Repo Discovery Page
Repo Discovery Page
│
├── Header
│   ├── Title: Repo Discovery
│   ├── Add Scan Folder Button
│   ├── Run Scan Button
│   └── Cancel Scan Button
│
├── Scan Sources Panel
│   ├── Common Folder Options
│   ├── Custom Folders
│   └── Excluded Folders
│
├── Suggested Repos List
│   ├── Repo Path
│   ├── Detected Remote
│   ├── Detected Package Manager
│   ├── Existing Managed Status
│   └── Checkbox
│
├── Bulk Actions Bar
│   ├── Select All
│   ├── Deselect All
│   ├── Approve Selected
│   └── Ignore Selected
│
└── Right Inspector
    └── Selected repo preview
Repo Discovery buttons
Button	Action
Add Scan Folder	Adds a custom local folder to scan
Run Scan	Searches configured folders for repos
Cancel Scan	Stops active scan
Approve Selected	Converts selected suggestions into managed repos
Ignore Selected	Hides selected suggestions from future discovery
Open in Files	Opens repo folder in Files workspace
Open in Finder	Reveals folder externally
Configure Profile	Opens repo profile setup
5.6 Managed Repositories Page
Managed Repositories
│
├── Header
│   ├── Search Repositories
│   ├── Filter by Profile
│   ├── Filter by Status
│   ├── Add Repo
│   └── Refresh All
│
├── Repo List / Table
│   ├── Repo Name
│   ├── Path
│   ├── Profile
│   ├── Current Branch
│   ├── Dirty Status
│   ├── Tests/Build Status
│   ├── Deployment Status
│   ├── Launch Gate Status
│   └── Last Activity
│
└── Right Inspector
    └── Selected repo summary
Repo row buttons
Button	Action
Open	Opens repo detail
View Status	Opens Git Status tab
View Diff	Opens Diff Viewer
Run Checks	Runs configured lint/test/build
Start Agent	Starts agent with repo context
Launch Gate	Opens repo Launch Gate
Open in Files	Opens repo folder in Files
Reveal in Finder	Opens system folder location
5.7 Repository Detail Layout
Repository Detail
│
├── Detail Header
│   ├── Repo Name
│   ├── Repo Path
│   ├── Current Branch
│   ├── Managed Status
│   ├── Start Agent Button
│   ├── Run Checks Button
│   ├── Create PR Button
│   ├── Open Launch Gate Button
│   └── More Menu
│
├── Detail Tabs
│   ├── Overview
│   ├── Git Status
│   ├── Changed Files
│   ├── Diff
│   ├── Branches
│   ├── Commits
│   ├── Pull Requests
│   ├── Deployments
│   ├── Launch Gate
│   ├── Linked Items
│   └── History
│
├── Selected Tab Content
│
└── Right Inspector
    └── Repo metadata, profile, links, review state, events
5.8 Repo Overview Tab
Repo Overview
│
├── Repo Profile Summary
├── Current Git State
├── Current Branch / Remote
├── Latest Commit
├── Open PRs
├── Active Deployments
├── Launch Gate Summary
├── Linked Product
├── Linked Tasks
├── Linked Agent Runs
└── Recent Repo Events
Buttons
Button	Action
Edit Repo Profile	Opens repo profile edit form
Link Product	Links repo to product
New Task	Creates task linked to repo
Start Agent	Starts Builder or Reviewer profile
Open Terminal Session	Opens repo working directory in session
Refresh Repo	Reloads Git state
5.9 Git Status Tab
Git Status
│
├── Working Tree Summary
├── Current Branch
├── Remote Tracking Status
├── Changed Files List
├── Untracked Files List
├── Staged Files List
├── Git Action Area
└── Status Events
Buttons
Button	Action
Refresh Status	Reloads git status
View Diff	Opens selected file diff
Open File	Opens selected file
Link File to Task	Links file to task
Create Review Task	Creates review task for current changes
Start Reviewer	Starts reviewer agent
Commit	Requires review/confirmation
Push	Requires confirmation
Discard Changes	Requires destructive confirmation
5.10 Diff Viewer Tab
Diff Viewer
│
├── File Selector
├── Diff Summary
├── File Diff Body
├── Comments / Review Notes Panel
├── Linked Task Panel
└── Actions Bar
Buttons
Button	Action
Previous File	Moves to previous changed file
Next File	Moves to next changed file
Open File	Opens file in Files or external editor
Add Review Note	Creates note/comment linked to file
Request Review	Starts review gate
Mark Reviewed	Marks file reviewed
Create Fix Task	Creates task from selected diff concern
5.11 Branches Tab
Branches
│
├── Current Branch Card
├── Local Branches List
├── Remote Branches List
└── Branch Actions
Buttons
Button	Action
Create Branch	Creates a new branch
Checkout	Switches branch with confirmation if dirty
Pull	Pulls latest changes
Push Branch	Pushes branch
Delete Branch	Requires confirmation
Compare	Opens diff/compare view
5.12 Commits Tab
Commits
│
├── Commit Timeline
├── Commit Detail Preview
├── Linked PR / Deployment
└── Commit Events
Buttons
Button	Action
Open Commit	Opens full commit detail
Copy SHA	Copies commit hash
Link to Task	Links commit to task
Link to Release	Links commit to product release
View Deployment	Opens deployment linked to commit
5.13 Pull Requests Tab
Pull Requests
│
├── PR Filters
├── PR List
├── PR Detail Preview
├── Checks Summary
├── Review Comments
└── Merge Area
Buttons
Button	Action
Create PR	Opens PR creation form
Open PR	Opens PR detail
Refresh PRs	Syncs GitHub PR data
Comment	Adds PR comment
Request Review	Starts review flow
Merge	Requires confirmation/review
Link Task	Links PR to task
Link Agent Run	Links PR to agent run
5.14 Vercel Deployments Tab
Vercel Deployments
│
├── Deployment Summary
├── Deployment List
├── Selected Deployment Detail
├── Build Logs
├── Preview / Production URLs
├── Verification Records
└── Deployment Actions
Buttons
Button	Action
Refresh Deployments	Reloads Vercel data
Open Preview URL	Opens preview in Browser workspace
Open Production URL	Opens production URL in Browser
View Logs	Opens build logs
Redeploy	Requires confirmation
Run Verification	Starts verification checks
Link to Launch Gate	Links deployment to Launch Gate
5.15 Launch Gate Page / Tab
Launch Gate
│
├── Gate Header
│   ├── State
│   ├── Repo
│   ├── Product
│   ├── Task
│   ├── Commit SHA
│   └── Deployment
│
├── Check List
│   ├── Working Tree Status
│   ├── Typecheck
│   ├── Lint
│   ├── Tests
│   ├── Build
│   ├── Review Approval
│   ├── Commit Pushed
│   ├── GitHub Checks
│   ├── Vercel Deploy Status
│   ├── Production URL HTTP Status
│   ├── Browser Console Errors
│   ├── Route Smoke Checks
│   ├── Asset Load Checks
│   └── Custom Verification Commands
│
├── Evidence Panel
├── Final Verdict Panel
├── History Timeline
└── Actions Footer
Launch Gate buttons
Button	Action
Run All Checks	Runs configured checks
Run Selected Check	Runs one check
Request Review	Starts reviewer agent
Deploy	Requires review/confirmation
Verify Production	Opens Browser verification flow
Mark Verified	Enabled only when evidence exists
Mark Failed	Records failure
Roll Back	Requires confirmation
Create Fix Task	Creates task from failed check
6. Agents Workspace
6.1 Purpose

Agents manages CLI-based AI workers, profiles, sessions, run history, blockers, and reviewer records.

6.2 Sitemap
Agents
├── Agents Dashboard
├── Active Runs
├── Agent Run History
│   └── Agent Run Detail
├── Agent Profiles
│   └── Agent Profile Detail/Edit
├── Reviewer Agent
├── Session Console
├── CLI Profiles
└── Agent Settings
6.3 Agents Dashboard Layout
Agents Dashboard
│
├── Workspace Header
│   ├── Title: Agents
│   ├── Search Runs
│   ├── New Agent Run Button
│   ├── New Agent Profile Button
│   ├── Open Session Console Button
│   └── Agent Settings Button
│
├── Summary Row
│   ├── Active Runs
│   ├── Blocked Runs
│   ├── Failed Runs
│   └── Pending Reviews
│
├── Main Body
│   ├── Active Runs List
│   ├── Recent Run History
│   ├── Agent Profiles List
│   └── Reviewer Records
│
└── Right Inspector
    └── Selected run/profile/session/review
6.4 Dashboard buttons
Button	Action
New Agent Run	Opens run setup dialog
New Agent Profile	Opens profile creation form
Open Session Console	Opens sessions panel/full page
Retry Failed	Retries selected failed run
Stop Run	Stops selected active run
Request Review	Starts reviewer agent for selected output
Create Task	Creates task linked to run
Agent Settings	Opens CLI/profile settings
6.5 New Agent Run Flow
New Agent Run Dialog
│
├── Agent Profile Selector
├── Mode Selector
├── Working Directory Selector
├── Prompt / Instruction Field
├── Context Attachments
│   ├── Task
│   ├── Repo
│   ├── Note
│   ├── Product
│   ├── Content Piece
│   ├── File
│   └── URL
├── Permission Preview
├── Review Requirement Preview
└── Footer Buttons
    ├── Cancel
    ├── Save as Draft
    └── Start Run
6.6 Active Runs Page
Active Runs
│
├── Toolbar
│   ├── Search
│   ├── Filter by Profile
│   ├── Filter by Workspace
│   └── Filter by Status
│
├── Active Run List
│   ├── Run Title
│   ├── Profile
│   ├── Linked Task
│   ├── Workspace
│   ├── Status
│   ├── Duration
│   ├── Last Output Summary
│   └── Attention State
│
└── Right Inspector
    └── Selected run detail
Active run buttons
Button	Action
Open	Opens run detail
Open Session	Opens live session panel
Stop	Stops active run
Send Input	Sends input into session
Attach Task	Links run to task
Mark Blocked	Marks run as blocked
Retry	Creates retry run
View Logs	Opens raw logs
6.7 Agent Run Detail Page
Agent Run Detail
│
├── Detail Header
│   ├── Run Title
│   ├── Profile
│   ├── Status
│   ├── Started / Completed Times
│   ├── Duration
│   ├── Open Session Button
│   ├── Retry Button
│   └── Request Review Button
│
├── Tabs
│   ├── Summary
│   ├── Prompt
│   ├── Clean Output
│   ├── Raw Logs
│   ├── Status Timeline
│   ├── Linked Entities
│   ├── Review
│   └── Events
│
└── Right Inspector
    └── Metadata, permissions, links, review state
Run detail buttons
Button	Action
Open Session	Opens session panel
Retry	Re-runs with same context
Duplicate Run	Creates a new run with same setup
Request Review	Starts reviewer profile
Create Follow-up Task	Creates task from output
Link Entity	Links output to task/product/repo/note/content
Export Logs	Saves logs to file
Mark Resolved	Resolves blocker/failure state
6.8 Agent Profiles Page
Agent Profiles
│
├── Toolbar
│   ├── Search Profiles
│   ├── New Profile Button
│   └── Import Profile Button
│
├── Profiles List
│   ├── Main Assistant
│   ├── Builder
│   ├── Reviewer
│   ├── Content
│   ├── Deployment Verifier
│   ├── Notes Organizer
│   └── Automation Runner
│
└── Right Inspector
    └── Profile configuration summary
6.9 Agent Profile Detail/Edit
Agent Profile Detail
│
├── Profile Header
│   ├── Name
│   ├── Enabled Status
│   ├── Test Profile Button
│   └── Save Button
│
├── Configuration Form
│   ├── Name
│   ├── Command
│   ├── Arguments
│   ├── Working Directory Behavior
│   ├── Environment Variable References
│   ├── Supported Modes
│   ├── Output Parser / Log Mode
│   ├── Status Detection
│   ├── Default Permissions
│   └── Reviewer Profile Flag
│
├── Recent Runs
└── Profile Events
Profile buttons
Button	Action
Save	Saves profile changes
Test Profile	Runs a test command
Duplicate	Copies profile
Disable	Disables profile
Delete	Requires confirmation
Set as Reviewer	Marks as reviewer profile
Open Runs	Filters run history by profile
6.10 Reviewer Agent Page
Reviewer Agent
│
├── Header
│   ├── Search Reviews
│   ├── New Review Button
│   └── Reviewer Profile Selector
│
├── Pending Review Queue
├── Completed Review Records
├── Required Fixes List
├── Blocked / Insufficient Evidence List
└── Right Inspector
    └── Selected ReviewRecord
Reviewer buttons
Button	Action
New Review	Opens review setup
Run Review	Starts reviewer agent
Approve	Records approved verdict
Require Fixes	Records required fixes verdict
Block	Records blocked verdict
Attach Evidence	Adds evidence
Open Reviewed Entity	Opens source object
Create Fix Task	Creates task from required fixes
7. Notes Workspace
7.1 Purpose

Notes replaces Apple Notes as the native knowledge system. Notes are Markdown-backed with metadata, links, tags, search, and import support.

7.2 Sitemap
Notes
├── Notes Dashboard
├── All Notes
│   └── Note Detail / Editor
├── Collections
│   └── Collection Detail
├── Tags
│   └── Tag Detail
├── Daily Notes
├── Code Notes
├── Content Notes
├── Business Notes
├── Product Notes
├── Imported Apple Notes
├── Backlinks
├── Entity-Linked Notes
└── Notes Settings
7.3 Notes Dashboard Layout
Notes Dashboard
│
├── Workspace Header
│   ├── Title: Notes
│   ├── Search Notes
│   ├── New Note Button
│   ├── New Collection Button
│   ├── Import Apple Notes Button
│   └── Notes Settings Button
│
├── Summary Row
│   ├── Recent Notes
│   ├── Unlinked Notes
│   ├── Imported Notes
│   └── Notes Needing Organization
│
├── Main Body
│   ├── Recent Notes List
│   ├── Collections List
│   ├── Tags List
│   ├── Linked Entity Notes
│   └── Recent Notes History
│
└── Right Inspector
    └── Selected note, collection, tag, or entity link
7.4 Notes buttons
Button	Action
New Note	Creates Markdown note
New Collection	Creates collection/folder
Import Apple Notes	Starts import flow through supported CLI
Search Notes	Searches title/body/metadata
Organize Notes	Starts Notes Organizer agent
Summarize	Runs summary on selected note
Link Entity	Links note to object
Delete Note	Requires confirmation
7.5 All Notes Page
All Notes
│
├── List Toolbar
│   ├── Search
│   ├── Filter by Collection
│   ├── Filter by Tag
│   ├── Filter by Workspace
│   ├── Sort
│   └── Bulk Actions
│
├── Notes List
│   ├── Title
│   ├── Collection
│   ├── Workspace
│   ├── Tags
│   ├── Last Updated
│   ├── Linked Entities Count
│   └── Summary
│
└── Right Inspector
    └── Selected note preview
Note row buttons
Button	Action
Open	Opens note editor
Quick Preview	Opens preview in inspector
Link	Opens entity link picker
Summarize	Runs summarizer
Move	Changes collection
Tag	Adds/removes tags
Delete	Requires confirmation
7.6 Note Detail / Editor Layout
Note Detail / Editor
│
├── Editor Header
│   ├── Note Title Field
│   ├── Save Status
│   ├── File Path
│   ├── Link Entity Button
│   ├── Summarize Button
│   ├── Organize Button
│   └── More Menu
│
├── Note Metadata Row
│   ├── Workspace Selector
│   ├── Collection Selector
│   ├── Tags
│   ├── Source
│   └── Updated Time
│
├── Editor Body
│   ├── Markdown Editor
│   └── Optional Preview Pane
│
├── Linked Entities Section
│   ├── Tasks
│   ├── Products
│   ├── Repos
│   ├── Content Pieces
│   ├── Contacts
│   ├── Companies
│   ├── Files
│   └── URLs
│
├── Backlinks Section
├── Note History Section
└── Right Inspector
    └── Metadata, links, events, file path
Editor buttons
Button	Action
Save	Saves note body and metadata
Link Entity	Links note to another object
Summarize	Runs AI/CLI summarizer
Organize	Suggests tags, collection, links
Create Task	Creates task from note
Open File	Opens Markdown file in Files
Reveal in Finder	Opens system location
Duplicate	Creates copy
Delete	Requires confirmation
7.7 Import Apple Notes Page
Imported Apple Notes
│
├── Header
│   ├── Start Import Button
│   ├── Select Import Destination
│   └── View Import Logs Button
│
├── Import Status Panel
│   ├── Pending
│   ├── In Progress
│   ├── Imported
│   ├── Failed
│   └── Skipped
│
├── Imported Notes List
├── Mapping / Organization Panel
└── Import History
Import buttons
Button	Action
Start Import	Starts Apple Notes import
Cancel Import	Stops active import
Retry Failed	Retries failed notes
Open Imported Note	Opens note
Organize Imported Notes	Runs organizer
Confirm Migration	Requires confirmation if bulk import affects data
8. Content Workspace
8.1 Purpose

Content handles MaVoid content planning, generation, assets, review, OmniSocials upload, scheduling, publishing, verification, and history.

8.2 Sitemap
Content
├── Content Dashboard
├── Content Calendar
├── Content Plans
│   └── Content Plan Detail
├── Content Pieces
│   └── Content Piece Detail
│       ├── Brief
│       ├── Draft
│       ├── Assets
│       ├── Captions
│       ├── Platform Adaptations
│       ├── Review
│       ├── Publishing
│       ├── Verification
│       └── History
├── AI Intel Brief Pipeline
├── Enterprise Carousel Pipeline
├── Asset Library
├── Review Queue
├── OmniSocials
├── Publishing History
├── Failure Reports
└── Content Settings
8.3 Content Dashboard Layout
Content Dashboard
│
├── Workspace Header
│   ├── Title: Content
│   ├── Search Content
│   ├── New Content Piece Button
│   ├── New Content Plan Button
│   ├── Generate Today’s Content Button
│   └── Content Settings Button
│
├── Summary Row
│   ├── Planned Today
│   ├── Drafts in Progress
│   ├── Needs Review
│   ├── Scheduled Posts
│   └── Publishing Failures
│
├── Main Body
│   ├── Content Calendar Preview
│   ├── Active Pipeline Items
│   ├── Review Queue
│   ├── OmniSocials Status
│   ├── Asset Queue
│   └── Recent Publishing History
│
└── Right Inspector
    └── Selected content piece, asset, review, schedule, or failure
8.4 Content buttons
Button	Action
New Content Piece	Creates content item
New Content Plan	Creates campaign/plan
Generate Today’s Content	Starts configured generation pipeline
Generate Brief	Starts AI Intel Brief pipeline
Generate Carousel	Starts enterprise carousel pipeline
Upload Media	Uploads asset to OmniSocials
Send to Review	Starts review gate
Schedule	Schedules post after review
Publish Now	Publishes immediately after confirmation
Verify	Verifies scheduled/published post
Create Failure Report	Records failed pipeline/publishing state
8.5 Content Calendar Page
Content Calendar
│
├── Header
│   ├── Calendar View Controls
│   ├── New Content Piece Button
│   ├── Generate Scheduled Content Button
│   └── Sync Publishing Status Button
│
├── Calendar Area
│   ├── Daily Slots
│   ├── Morning AI Intel Brief Slots
│   ├── Evening Enterprise Carousel Slots
│   ├── Scheduled Posts
│   ├── Failed Posts
│   └── Empty Slots
│
├── Queue Sidebar or Inspector
│   ├── Unscheduled Drafts
│   ├── Needs Review
│   ├── Ready to Schedule
│   └── Failed Publishing
│
└── Right Inspector
    └── Selected slot or content item
Calendar buttons
Button	Action
New Content Piece	Creates item for selected date/slot
Generate	Starts generation for selected slot
Reschedule	Changes date/time
Open Piece	Opens content piece detail
Review	Opens review tab
Upload Media	Starts media upload
Schedule	Sends schedule intent to OmniSocials
Verify	Verifies scheduled status
8.6 Content Plans Page
Content Plans
│
├── List Toolbar
│   ├── Search
│   ├── Filter by Status
│   ├── Filter by Pillar
│   ├── New Plan
│   └── Archive Plan
│
├── Content Plan List
│   ├── Plan Name
│   ├── Status
│   ├── Date Range
│   ├── Content Pieces Count
│   ├── Platforms
│   └── Last Activity
│
└── Right Inspector
    └── Selected plan summary
8.7 Content Plan Detail
Content Plan Detail
│
├── Detail Header
│   ├── Plan Name
│   ├── Status
│   ├── Date Range
│   ├── Generate Pieces Button
│   └── More Menu
│
├── Plan Sections
│   ├── Strategy / Objective
│   ├── Content Pillars
│   ├── Planned Pieces
│   ├── Calendar Placement
│   ├── Assets
│   ├── Reviews
│   ├── Publishing Records
│   └── History
│
└── Right Inspector
    └── Metadata and links
Plan buttons
Button	Action
Generate Pieces	Creates content pieces from plan
Add Piece	Adds manual piece
Link Product	Links plan to product
Link Notes	Links supporting notes
Archive Plan	Archives plan
Create Task	Creates task linked to plan
8.8 Content Pieces Page
Content Pieces
│
├── List Toolbar
│   ├── Search
│   ├── Filter by Status
│   ├── Filter by Type
│   ├── Filter by Platform
│   ├── Filter by Review State
│   └── New Content Piece
│
├── Content Piece List
│   ├── Title
│   ├── Type
│   ├── Status
│   ├── Platform Targets
│   ├── Scheduled Time
│   ├── Review State
│   ├── Publishing State
│   └── Last Activity
│
└── Right Inspector
    └── Selected content piece preview
8.9 Content Piece Detail Layout
Content Piece Detail
│
├── Detail Header
│   ├── Title
│   ├── Type
│   ├── Status
│   ├── Target Platforms
│   ├── Generate Button
│   ├── Review Button
│   ├── Schedule Button
│   └── More Menu
│
├── Pipeline Status Bar
│   ├── Plan
│   ├── Generate
│   ├── Design / Assets
│   ├── Review
│   ├── Upload Media
│   ├── Schedule / Publish
│   ├── Verify
│   └── Record
│
├── Detail Tabs
│   ├── Brief
│   ├── Draft
│   ├── Assets
│   ├── Captions
│   ├── Platform Adaptations
│   ├── Review
│   ├── Publishing
│   ├── Verification
│   └── History
│
└── Right Inspector
    └── Metadata, status, linked plan, linked assets, policy, events
Content piece buttons
Button	Action
Generate Draft	Starts content generation session
Generate Assets	Starts asset generation session
Attach Asset	Links media asset
Send to Review	Starts reviewer gate
Approve	Records approval
Require Fixes	Records fixes needed
Upload Media	Uploads to OmniSocials
Schedule	Schedules post after approval
Publish Now	Publishes immediately after confirmation
Verify Scheduled Post	Confirms schedule status
Verify Published Post	Confirms published status
Create Failure Report	Records failure
Duplicate Piece	Creates copy
Archive	Archives item
8.10 Brief Tab
Brief Tab
│
├── Content Objective
├── Audience
├── Platform Targets
├── Format
├── Source Context
├── Linked Notes / Files / URLs
├── Constraints
└── Buttons
    ├── Save Brief
    ├── Generate Draft
    ├── Attach Source
    └── Link Entity
8.11 Draft Tab
Draft Tab
│
├── Draft Editor
├── Version History
├── AI Generation Session Link
├── Notes / Comments
└── Buttons
    ├── Save Draft
    ├── Regenerate
    ├── Create Variant
    ├── Send to Review
    └── Create Task
8.12 Assets Tab
Assets Tab
│
├── Asset Requirements
├── Linked Media Assets
├── Carousel Item List
├── Asset File Preview
├── Upload Readiness Checklist
└── Buttons
    ├── Generate Assets
    ├── Attach Existing Asset
    ├── Open Asset
    ├── Replace Asset
    ├── Remove Asset
    └── Upload Media
8.13 Captions Tab
Captions Tab
│
├── Master Caption
├── Platform-Specific Captions
├── Hashtags / Mentions
├── CTA Field
├── Caption Version History
└── Buttons
    ├── Generate Caption
    ├── Adapt per Platform
    ├── Save
    ├── Send to Review
    └── Copy Caption
8.14 Platform Adaptations Tab
Platform Adaptations
│
├── Instagram Adaptation
│   ├── Media Required Check
│   ├── Carousel Count Check
│   └── Caption
├── LinkedIn Adaptation
├── Facebook Adaptation
├── X/Twitter Adaptation
│   ├── Media Count Check
│   └── Adaptation Recommendation
└── Buttons
    ├── Generate Adaptations
    ├── Validate Constraints
    ├── Disable Platform
    └── Save Adaptations
8.15 Review Tab
Review Tab
│
├── Review Requirement Summary
├── Reviewer Agent Selector
├── Evidence Package
│   ├── Brief
│   ├── Draft
│   ├── Assets
│   ├── Captions
│   └── Platform Constraints
├── Review Result
│   ├── Verdict
│   ├── Evidence Summary
│   └── Required Fixes
└── Buttons
    ├── Run Review
    ├── Approve
    ├── Require Fixes
    ├── Create Fix Task
    └── Re-run After Fixes
8.16 Publishing Tab
Publishing Tab
│
├── OmniSocials Connection Status
├── Target Accounts
├── Media Upload Status
├── Schedule Time
├── Publishing Mode
│   ├── Schedule
│   └── Publish Now
├── Platform Validation Results
├── Confirmation Area
└── Buttons
    ├── Connect OmniSocials
    ├── Upload Media
    ├── Validate
    ├── Schedule
    ├── Publish Now
    └── Cancel Scheduled Post
8.17 Verification Tab
Verification Tab
│
├── Scheduled Post Record
├── Published Post Record
├── Verification Checklist
│   ├── Accounts
│   ├── Media
│   ├── Schedule Time
│   ├── Platform Status
│   └── Publish URL if available
├── Verification Evidence
└── Buttons
    ├── Verify Scheduled Post
    ├── Verify Published Post
    ├── Record Success
    ├── Record Failure
    └── Create Failure Report
8.18 Asset Library Page
Asset Library
│
├── Toolbar
│   ├── Search Assets
│   ├── Filter by Type
│   ├── Filter by Linked Content
│   ├── Add Asset
│   └── Open Assets Folder
│
├── Asset Grid/List
│   ├── Asset Name
│   ├── Type
│   ├── Linked Content Piece
│   ├── File Path
│   ├── Upload Status
│   └── Last Used
│
└── Right Inspector
    └── Selected asset details
Asset buttons
Button	Action
Add Asset	Selects local media
Generate Asset	Starts generation session
Open	Opens preview
Link to Content	Links asset
Upload	Uploads to OmniSocials
Reveal in Files	Opens file in Files workspace
Delete	Requires confirmation
9. Automations Workspace
9.1 Purpose

Automations manages scheduled and manual automation workflows, run logs, failures, policies, and linked entities.

9.2 Sitemap
Automations
├── Automations Dashboard
├── All Automations
│   └── Automation Detail
│       ├── Overview
│       ├── Schedule
│       ├── Runs
│       ├── Logs
│       ├── Linked Entities
│       ├── Failure Policy
│       └── History
├── Automation Runs
│   └── Automation Run Detail
├── Failed Runs
├── Schedules
├── Default Automations
└── Automation Settings
9.3 Automations Dashboard Layout
Automations Dashboard
│
├── Workspace Header
│   ├── Title: Automations
│   ├── Search Automations
│   ├── New Automation Button
│   ├── Run Selected Button
│   └── Automation Settings Button
│
├── Summary Row
│   ├── Enabled Automations
│   ├── Running Now
│   ├── Failed Last Runs
│   └── Upcoming Runs
│
├── Main Body
│   ├── Automation List
│   ├── Failed Runs
│   ├── Upcoming Schedule
│   ├── Recent Runs
│   └── Default Automations Panel
│
└── Right Inspector
    └── Selected automation or run
9.4 Automation buttons
Button	Action
New Automation	Creates automation
Run Now	Starts automation manually
Pause	Pauses enabled automation
Resume	Re-enables paused automation
Retry Failed Run	Starts retry from failed run
Change Schedule	Opens schedule editor
View Logs	Opens logs
Disable Publishing Only	Disables publishing step while preserving drafts if policy supports it
Disable Automation	Requires confirmation
Delete Automation	Requires confirmation
9.5 All Automations Page
All Automations
│
├── List Toolbar
│   ├── Search
│   ├── Filter by Workspace
│   ├── Filter by Type
│   ├── Filter by Enabled Status
│   ├── Filter by Health
│   └── New Automation
│
├── Automation List
│   ├── Name
│   ├── Workspace
│   ├── Type
│   ├── Schedule
│   ├── Enabled
│   ├── Status
│   ├── Last Run
│   ├── Next Run
│   └── Last Result
│
└── Right Inspector
    └── Automation preview
9.6 Automation Detail Page
Automation Detail
│
├── Detail Header
│   ├── Name
│   ├── Enabled Status
│   ├── Current Status
│   ├── Run Now Button
│   ├── Pause / Resume Button
│   └── More Menu
│
├── Tabs
│   ├── Overview
│   ├── Schedule
│   ├── Runs
│   ├── Logs
│   ├── Linked Entities
│   ├── Failure Policy
│   └── History
│
└── Right Inspector
    └── Metadata, policy, permissions, events
9.7 Automation Overview Tab
Overview
│
├── Automation Summary
├── Type
├── Workspace
├── Enabled Status
├── Last Run Result
├── Next Run
├── Linked Entities
├── Notification Policy
└── Recent Events
Buttons
Button	Action
Edit Automation	Opens edit mode
Run Now	Manual run
Pause / Resume	Toggles enabled state
Create Task	Creates task linked to automation
Open Last Run	Opens latest AutomationRun
View Failures	Filters failed runs
9.8 Schedule Tab
Schedule
│
├── Schedule Editor
│   ├── Trigger Type
│   ├── Time / Interval
│   ├── Timezone
│   ├── Conditions
│   └── Next Run Preview
├── Conflict / Risk Warnings
└── Buttons
    ├── Save Schedule
    ├── Test Schedule
    ├── Disable Schedule
    └── Reset Schedule
9.9 Runs Tab
Runs
│
├── Runs Toolbar
│   ├── Filter by Status
│   ├── Filter by Trigger
│   └── Search Logs Summary
├── Run List
│   ├── Started
│   ├── Duration
│   ├── Status
│   ├── Trigger
│   ├── Output Summary
│   └── Error Summary
└── Right Inspector
    └── Selected run detail
Run buttons
Button	Action
Open Run	Opens run detail
Retry	Retries run
View Logs	Opens logs
Create Fix Task	Creates task from failure
Link Entity	Links run to object
9.10 Automation Run Detail
Automation Run Detail
│
├── Detail Header
│   ├── Automation Name
│   ├── Run Status
│   ├── Started / Completed
│   ├── Duration
│   ├── Retry Button
│   └── View Logs Button
│
├── Sections
│   ├── Trigger
│   ├── Inputs
│   ├── Outputs
│   ├── Errors
│   ├── Logs
│   ├── Linked Task
│   ├── Linked Entities
│   └── Events
│
└── Right Inspector
    └── Run metadata
10. Business Workspace
10.1 Purpose

Business is a lightweight CRM and operations area for contacts, companies, clients, follow-ups, linked Gmail, tasks, notes, products, and SOP notes.

10.2 Sitemap
Business
├── Business Dashboard
├── Contacts
│   └── Contact Detail
├── Companies / Clients
│   └── Company Detail
├── Follow-ups
├── Linked Gmail
├── Business Tasks
├── SOP Notes
└── Business Settings
10.3 Business Dashboard Layout
Business Dashboard
│
├── Workspace Header
│   ├── Title: Business
│   ├── Search Business
│   ├── New Contact Button
│   ├── New Company Button
│   ├── New Follow-up Button
│   └── Business Settings Button
│
├── Summary Row
│   ├── Follow-ups Due
│   ├── Recent Gmail Threads
│   ├── Active Clients / Companies
│   └── Business Tasks
│
├── Main Body
│   ├── Contacts Preview
│   ├── Companies Preview
│   ├── Follow-ups List
│   ├── Linked Emails
│   ├── Business Notes / SOPs
│   └── Recent Business History
│
└── Right Inspector
    └── Selected contact, company, email, follow-up, or task
10.4 Business buttons
Button	Action
New Contact	Creates contact
New Company	Creates company/client
New Follow-up	Creates follow-up task/calendar item
Link Gmail Thread	Links email thread
Draft Email	Opens email composer
Create Task	Creates business task
Add Note	Creates note linked to contact/company
Link Product	Links company/contact to product
10.5 Contacts Page
Contacts
│
├── List Toolbar
│   ├── Search Contacts
│   ├── Filter by Company
│   ├── Filter by Tag
│   ├── Sort
│   └── New Contact
│
├── Contacts List
│   ├── Name
│   ├── Company
│   ├── Role
│   ├── Email
│   ├── Phone
│   ├── Tags
│   ├── Follow-up Status
│   └── Last Activity
│
└── Right Inspector
    └── Contact preview
Contact row buttons
Button	Action
Open	Opens contact detail
Email	Opens compose/draft flow
New Follow-up	Creates follow-up
Add Note	Creates linked note
Create Task	Creates linked task
Link Product	Links product
Delete	Requires confirmation
10.6 Contact Detail Page
Contact Detail
│
├── Detail Header
│   ├── Contact Name
│   ├── Company
│   ├── Role
│   ├── Email Button
│   ├── New Follow-up Button
│   └── More Menu
│
├── Tabs
│   ├── Overview
│   ├── Emails
│   ├── Tasks
│   ├── Notes
│   ├── Products
│   ├── Follow-ups
│   └── History
│
└── Right Inspector
    └── Metadata, source, linked entities, recent events
10.7 Contact Overview Tab
Overview
│
├── Contact Information
│   ├── Name
│   ├── Email
│   ├── Phone
│   ├── Company
│   ├── Role
│   └── Tags
├── Relationship Summary
├── Recent Emails
├── Open Tasks
├── Upcoming Follow-ups
├── Linked Notes
└── Recent History
Contact detail buttons
Button	Action
Edit Contact	Opens edit form
Draft Email	Creates draft
New Follow-up	Creates follow-up
Add Note	Creates linked note
Create Task	Creates linked task
Link Email	Links Gmail thread/message
Link Product	Links product
Archive Contact	Archives contact
Delete Contact	Requires confirmation
10.8 Companies / Clients Page
Companies / Clients
│
├── List Toolbar
│   ├── Search Companies
│   ├── Filter by Status
│   ├── Filter by Tag
│   └── New Company
│
├── Companies List
│   ├── Company Name
│   ├── Status
│   ├── Primary Contacts
│   ├── Open Tasks
│   ├── Follow-ups
│   ├── Linked Products
│   └── Last Activity
│
└── Right Inspector
    └── Company preview
10.9 Company Detail Page
Company Detail
│
├── Detail Header
│   ├── Company Name
│   ├── Status
│   ├── New Contact Button
│   ├── New Follow-up Button
│   └── More Menu
│
├── Tabs
│   ├── Overview
│   ├── Contacts
│   ├── Emails
│   ├── Tasks
│   ├── Notes
│   ├── Products
│   ├── Follow-ups
│   └── History
│
└── Right Inspector
    └── Company metadata and linked entities
Company detail buttons
Button	Action
Edit Company	Opens edit form
New Contact	Creates contact linked to company
Draft Email	Starts email draft
New Follow-up	Creates follow-up
Add Note	Creates linked note
Create Task	Creates linked task
Link Product	Links product
Archive Company	Archives company
Delete Company	Requires confirmation
10.10 Follow-ups Page
Follow-ups
│
├── Toolbar
│   ├── Search
│   ├── Filter by Due Date
│   ├── Filter by Contact
│   ├── Filter by Company
│   ├── Filter by Status
│   └── New Follow-up
│
├── Follow-up List
│   ├── Title
│   ├── Contact / Company
│   ├── Due Date
│   ├── Status
│   ├── Linked Email
│   ├── Linked Task
│   └── Notes
│
└── Right Inspector
    └── Selected follow-up
Follow-up buttons
Button	Action
Open	Opens follow-up detail
Mark Done	Completes follow-up
Snooze	Changes due date
Draft Email	Starts email draft
Create Calendar Event	Requires confirmation
Link Email	Links Gmail message/thread
Convert to Task	Creates or links task
11. Products Workspace
11.1 Purpose

Products is the operating layer for products like Leadra, Zoid, Zoid AI, MaVoid Unified Platform, MaVoid systems, and client products.

11.2 Sitemap
Products
├── Products Dashboard
├── All Products
│   └── Product Detail
│       ├── Overview
│       ├── Roadmap
│       ├── Tasks
│       ├── Repos
│       ├── Deployments
│       ├── Releases
│       ├── Notes / Decisions
│       ├── Content
│       ├── Automations
│       ├── Clients / Companies
│       └── Timeline / History
├── Roadmap Items
├── Product Launches
├── Product Decisions
└── Product Settings
11.3 Products Dashboard Layout
Products Dashboard
│
├── Workspace Header
│   ├── Title: Products
│   ├── Search Products
│   ├── New Product Button
│   ├── New Roadmap Item Button
│   └── Product Settings Button
│
├── Summary Row
│   ├── Active Products
│   ├── Products Needing Attention
│   ├── Upcoming Launches
│   └── Active Product Tasks
│
├── Main Body
│   ├── Product List
│   ├── Roadmap Preview
│   ├── Launch Timeline
│   ├── Product Tasks
│   ├── Linked Deployments
│   └── Product History
│
└── Right Inspector
    └── Selected product, roadmap item, release, task, or deployment
11.4 Products buttons
Button	Action
New Product	Creates product
New Roadmap Item	Creates roadmap item
Create Task	Creates product task
Link Repo	Links repository
Link Content	Links content piece
Add Decision	Creates decision note
Run Launch Gate	Starts product launch gate flow
View Timeline	Opens product history
11.5 All Products Page
All Products
│
├── List Toolbar
│   ├── Search
│   ├── Filter by Status
│   ├── Filter by Type
│   ├── Sort
│   └── New Product
│
├── Product List
│   ├── Name
│   ├── Status
│   ├── Type
│   ├── Owner
│   ├── Linked Repos
│   ├── Open Tasks
│   ├── Active Deployments
│   ├── Upcoming Releases
│   └── Last Activity
│
└── Right Inspector
    └── Product preview
11.6 Product Detail Layout
Product Detail
│
├── Detail Header
│   ├── Product Name
│   ├── Status
│   ├── Type
│   ├── Owner
│   ├── Create Task Button
│   ├── Link Repo Button
│   ├── Run Launch Gate Button
│   └── More Menu
│
├── Tabs
│   ├── Overview
│   ├── Roadmap
│   ├── Tasks
│   ├── Repos
│   ├── Deployments
│   ├── Releases
│   ├── Notes / Decisions
│   ├── Content
│   ├── Automations
│   ├── Clients / Companies
│   └── Timeline / History
│
└── Right Inspector
    └── Product metadata, links, review/deployment status, events
11.7 Product Overview Tab
Overview
│
├── Product Summary
├── Status
├── Description
├── Owner
├── Linked Repos
├── Open Tasks
├── Current Deployments
├── Upcoming Releases
├── Recent Decisions
├── Linked Content
├── Active Automations
└── Recent History
Product overview buttons
Button	Action
Edit Product	Opens edit form
Create Task	Creates linked task
Link Repo	Links repository
Link Note	Links note
Add Decision	Creates decision note
Add Roadmap Item	Creates roadmap item
Create Release	Creates release record
Run Agent	Starts agent with product context
11.8 Roadmap Tab
Roadmap
│
├── Roadmap Toolbar
│   ├── Filter by Status
│   ├── Filter by Priority
│   ├── New Roadmap Item
│   └── View Toggle
│
├── Roadmap Items
│   ├── Planned
│   ├── In Progress
│   ├── Blocked
│   ├── Needs Review
│   └── Done
│
└── Right Inspector
    └── Selected roadmap item
Roadmap buttons
Button	Action
New Roadmap Item	Creates item
Open	Opens item
Create Task	Creates task from item
Link Repo	Links repo
Link Note	Links decision/note
Mark In Progress	Updates status
Mark Done	Completes item
11.9 Repos Tab
Repos
│
├── Linked Repos List
├── Repo Status Summary
├── Dirty Repo Alerts
├── Launch Gate States
└── Buttons
    ├── Link Repo
    ├── Open Repo
    ├── Run Checks
    ├── Start Agent
    └── Open Launch Gate
11.10 Deployments / Releases Tabs
Deployments
│
├── Active Deployments
├── Production URLs
├── Preview URLs
├── Verification Records
└── Buttons
    ├── Open Deployment
    ├── Verify
    ├── Link Launch Gate
    └── Create Release

Releases
│
├── Release List
├── Release Detail Preview
├── Linked Commits / PRs
├── Linked Launch Gates
├── Verification Evidence
└── Buttons
    ├── New Release
    ├── Link Commit
    ├── Link Deployment
    ├── Mark Released
    └── Create Post-Release Task
12. Files Workspace
12.1 Purpose

Files is a Zoid-aware file manager for browsing local folders, previewing files, moving/renaming/copying/deleting with confirmation, and linking files to Zoid entities.

12.2 Sitemap
Files
├── Files Dashboard
├── File Explorer
│   └── File / Folder Detail
├── Pinned Folders
├── Zoid Folder
│   ├── Notes
│   ├── Content
│   ├── Assets
│   ├── Exports
│   ├── Imports
│   ├── Files
│   └── Products
├── Repositories
├── Content Assets
├── Browser Captures
├── Imports
├── Exports
├── Linked Files
└── File Settings
12.3 Files Dashboard Layout
Files Dashboard
│
├── Workspace Header
│   ├── Title: Files
│   ├── Search Files
│   ├── Add Folder Button
│   ├── New Folder Button
│   └── File Settings Button
│
├── Summary Row
│   ├── Pinned Folders
│   ├── Zoid Folder
│   ├── Recent Files
│   └── Linked Files
│
├── Main Body
│   ├── Folder Shortcuts
│   ├── Recent Files List
│   ├── Content Assets
│   ├── Browser Captures
│   ├── Repo Folders
│   └── File Events
│
└── Right Inspector
    └── Selected file or folder
12.4 File Explorer Layout
File Explorer
│
├── Header / Path Bar
│   ├── Back Button
│   ├── Forward Button
│   ├── Up Folder Button
│   ├── Breadcrumb Path
│   ├── Search Current Folder
│   ├── New Folder Button
│   └── More Menu
│
├── Left Folder Tree
│   ├── Pinned Folders
│   ├── Zoid Folder
│   ├── Repositories
│   ├── Content Assets
│   ├── Imports
│   └── Exports
│
├── Main File List
│   ├── Name
│   ├── Kind
│   ├── Size
│   ├── Modified
│   ├── Linked Entity
│   └── Repo Status if applicable
│
├── Preview Pane
│   ├── Markdown Preview
│   ├── Text / Code Preview
│   ├── Image Preview
│   ├── PDF Preview if feasible
│   └── Basic Metadata
│
└── Right Inspector
    └── File metadata, links, events, actions
12.5 File buttons
Button	Action
Open	Opens file
Preview	Opens preview pane
Rename	Renames file/folder
Move	Opens move picker
Copy	Copies file/folder
Duplicate	Creates duplicate
Delete	Moves to trash after confirmation
New Folder	Creates folder
Reveal in Finder	Opens system location
Link to Entity	Links file/folder to task/note/product/content/repo/contact
Open in Code Workspace	Opens repo-aware file context
Open in Notes	Opens Markdown note if applicable
Copy Path	Copies local path
12.6 File / Folder Detail Inspector
File / Folder Inspector
│
├── Header
│   ├── Name
│   ├── Type
│   ├── Path
│   └── Quick Actions
│
├── Metadata
│   ├── Size
│   ├── Created
│   ├── Modified
│   ├── Extension
│   └── Source
│
├── Preview
├── Linked Entities
│   ├── Tasks
│   ├── Notes
│   ├── Products
│   ├── Content Pieces
│   ├── Agent Runs
│   └── Events
├── Repo-Aware Section if inside repo
│   ├── Repo Name
│   ├── Git Status
│   ├── Branch
│   └── Related Diff
└── Actions
    ├── Open
    ├── Rename
    ├── Move
    ├── Copy
    ├── Duplicate
    ├── Delete
    ├── Reveal in Finder
    └── Link to Entity
13. Browser Workspace
13.1 Purpose

Browser is a work-only browser for research, app verification, deployed app checks, saved pages, screenshots, and linking pages to Zoid entities.

13.2 Sitemap
Browser
├── Browser Workspace
├── Active Tabs
├── Pinned Tabs
├── Work Bookmarks
├── Work History
├── Saved Pages
├── Screenshots / Captures
├── App Verification Sessions
└── Browser Settings
13.3 Browser Workspace Layout
Browser Workspace
│
├── Browser Header
│   ├── Back Button
│   ├── Forward Button
│   ├── Reload Button
│   ├── Address / Search Field
│   ├── New Tab Button
│   ├── Save Page Button
│   ├── Screenshot Button
│   ├── Link Entity Button
│   └── More Menu
│
├── Tab Bar
│   ├── Active Tab
│   ├── Other Tabs
│   └── Pinned Tabs
│
├── Webview Area
│   └── Current Page
│
├── Optional Verification Panel
│   ├── URL Status
│   ├── Console Errors
│   ├── Screenshot Evidence
│   ├── Route Checks
│   └── Asset Checks
│
└── Right Inspector
    └── Page metadata, captures, links, notes, verification evidence
13.4 Browser buttons
Button	Action
Back	Navigates back
Forward	Navigates forward
Reload	Reloads page
New Tab	Opens new tab
Close Tab	Closes current tab
Pin Tab	Pins current tab
Bookmark	Saves work bookmark
Save Page to Note	Creates note from page
Summarize Page	Runs configured CLI/agent
Extract Key Points	Creates extraction summary
Save Screenshot	Captures page screenshot
Link Page to Entity	Links tab/page to task/note/product/content
Start Verification	Begins app verification flow
Capture Console	Captures console/error data if feasible
13.5 Saved Pages Page
Saved Pages
│
├── Toolbar
│   ├── Search Saved Pages
│   ├── Filter by Linked Entity
│   ├── Filter by Source
│   └── New Capture
│
├── Saved Pages List
│   ├── Title
│   ├── URL
│   ├── Saved At
│   ├── Linked Entities
│   ├── Screenshot Status
│   └── Summary
│
└── Right Inspector
    └── Selected saved page
13.6 Screenshots / Captures Page
Screenshots / Captures
│
├── Toolbar
│   ├── Search
│   ├── Filter by Entity
│   ├── Filter by Date
│   └── Open Captures Folder
│
├── Captures Grid/List
│   ├── Capture Name
│   ├── Source URL
│   ├── Captured At
│   ├── Linked Entity
│   └── File Path
│
└── Right Inspector
    └── Capture preview and links
13.7 App Verification Session Page
App Verification Session
│
├── Session Header
│   ├── Target URL
│   ├── Linked Product
│   ├── Linked Repo
│   ├── Linked Launch Gate
│   └── Run Verification Button
│
├── Verification Checklist
│   ├── HTTP Status
│   ├── Route Smoke Checks
│   ├── Console Errors
│   ├── Asset Load Checks
│   ├── Screenshot Evidence
│   └── Custom Checks
│
├── Evidence List
├── Results Summary
└── Buttons
    ├── Run Checks
    ├── Save Evidence
    ├── Link to Launch Gate
    ├── Mark Passed
    ├── Mark Failed
    └── Create Fix Task
14. Inbox Workspace
14.1 Purpose

Inbox is the unified attention center for notifications, blockers, approvals, required fixes, publishing status, reminders, and Gmail.

14.2 Sitemap
Inbox
├── Inbox Dashboard
├── All Attention Items
├── Zoid Notifications
├── Agent Completions
├── Agent Blockers
├── Automation Failures
├── Review Approvals
├── Required Fixes
├── Content Publishing Status
├── Calendar / Task Reminders
├── Gmail
│   ├── Recent Messages
│   ├── Search Messages
│   ├── Thread Detail
│   ├── Draft Reply
│   └── Compose Email
└── Inbox Settings
14.3 Inbox Dashboard Layout
Inbox Dashboard
│
├── Workspace Header
│   ├── Title: Inbox
│   ├── Search Inbox
│   ├── Connect Gmail Button if disconnected
│   ├── Compose Email Button
│   ├── Mark Selected Resolved Button
│   └── Inbox Settings Button
│
├── Summary Row
│   ├── Urgent Attention Items
│   ├── Pending Approvals
│   ├── Failed Automations
│   ├── Agent Blockers
│   └── Gmail Items
│
├── Main Body
│   ├── Attention Feed
│   ├── Approval Queue
│   ├── Gmail Summary
│   ├── Calendar / Task Reminders
│   ├── Content Publishing Status
│   └── Recent Notifications
│
└── Right Inspector
    └── Selected notification, email, review, failure, or reminder
14.4 Inbox buttons
Button	Action
Open	Opens linked entity
Resolve	Marks notification/action item resolved
Snooze	Delays item
Create Task	Creates task from item
Link Entity	Links notification/email to object
Approve	Approves pending action where allowed
Reject / Require Fixes	Records required fixes or rejection
View Evidence	Opens review/deployment/publishing evidence
Retry	Retries failed automation/agent/publishing step
Compose Email	Opens email composer
Draft Reply	Creates draft reply
Send	Sends email after confirmation
14.5 All Attention Items Page
All Attention Items
│
├── Toolbar
│   ├── Search
│   ├── Filter by Workspace
│   ├── Filter by Severity
│   ├── Filter by Type
│   ├── Filter by Status
│   └── Bulk Actions
│
├── Attention Item List
│   ├── Title
│   ├── Type
│   ├── Workspace
│   ├── Severity
│   ├── Linked Entity
│   ├── Created At
│   └── Status
│
└── Right Inspector
    └── Selected attention item
14.6 Review Approvals Page
Review Approvals
│
├── Toolbar
│   ├── Filter by Verdict
│   ├── Filter by Entity Type
│   ├── Filter by Workspace
│   └── Search Reviews
│
├── Review Queue
│   ├── Reviewed Entity
│   ├── Proposed Action
│   ├── Reviewer Verdict
│   ├── Evidence Summary
│   ├── Required Fixes
│   └── Created At
│
└── Right Inspector
    └── Selected review record
Review approval buttons
Button	Action
Open Review	Opens ReviewRecord
Open Entity	Opens reviewed item
View Evidence	Opens evidence panel
Approve and Continue	Allows action to proceed
Require Fixes	Blocks action and creates fixes
Create Fix Task	Creates task
Re-run Review	Starts reviewer again
14.7 Gmail Page
Gmail
│
├── Gmail Header
│   ├── Connection Status
│   ├── Search Messages
│   ├── Compose Button
│   ├── Refresh Button
│   └── Gmail Settings Button
│
├── Message List
│   ├── Sender
│   ├── Subject
│   ├── Snippet
│   ├── Date
│   ├── Attachment Indicator
│   └── Linked Entity Indicator
│
├── Thread Preview / Detail
│   ├── Thread Messages
│   ├── Summary
│   ├── Linked Contacts / Companies
│   ├── Linked Tasks
│   └── Draft Reply Area
│
└── Right Inspector
    └── Email metadata, links, contact/company matching
Gmail buttons
Button	Action
Connect Gmail	Starts Gmail auth
Search	Searches Gmail messages
Refresh	Reloads recent messages
Open Thread	Opens full thread
Summarize Thread	Runs configured summarizer
Draft Reply	Creates draft reply
Compose	Opens compose form
Send	Sends after confirmation
Link to Contact	Links email to contact
Link to Company	Links email to company
Convert to Task	Creates task
Create Follow-up	Creates follow-up
Attachments	Requires confirmation before handling attachments
14.8 Compose Email Layout
Compose Email
│
├── Compose Header
│   ├── Draft Status
│   ├── Linked Entity
│   └── Close Button
│
├── Fields
│   ├── To
│   ├── CC
│   ├── BCC
│   ├── Subject
│   └── Body
│
├── Context Panel
│   ├── Linked Contact
│   ├── Linked Company
│   ├── Related Notes
│   ├── Related Tasks
│   └── Related Email Thread
│
└── Footer Buttons
    ├── Save Draft
    ├── Discard Draft
    ├── Request Review
    └── Send after Confirmation
15. Shared Tasks System

Tasks are first-class objects and should appear across every workspace.

15.1 Sitemap
Tasks
├── Task Inbox
├── Todo
├── In Progress
├── Waiting / Blocked
├── Needs Review
├── Done
├── Archived
└── Task Detail
15.2 Global Tasks Layout
Tasks Page
│
├── Header
│   ├── Search Tasks
│   ├── New Task Button
│   ├── Filter Button
│   ├── Sort Button
│   └── View Toggle
│
├── Status Navigation
│   ├── Inbox
│   ├── Todo
│   ├── In Progress
│   ├── Waiting / Blocked
│   ├── Needs Review
│   ├── Done
│   └── Archived
│
├── Task List / Board
│   ├── Task Title
│   ├── Workspace
│   ├── Status
│   ├── Priority
│   ├── Due Date
│   ├── Linked Entities
│   └── Last Activity
│
└── Right Inspector
    └── Selected task details
15.3 Task Detail Layout
Task Detail
│
├── Detail Header
│   ├── Task Title
│   ├── Status
│   ├── Priority
│   ├── Due Date
│   ├── Start Agent Button
│   ├── Mark Done Button
│   └── More Menu
│
├── Main Sections
│   ├── Description
│   ├── Checklist / Subtasks
│   ├── Linked Entities
│   │   ├── Repo
│   │   ├── Note
│   │   ├── Content Piece
│   │   ├── Content Plan
│   │   ├── Agent Run
│   │   ├── Deployment
│   │   ├── Contact / Company
│   │   ├── Automation
│   │   ├── Decision
│   │   ├── File Path
│   │   └── URL
│   ├── Agent Runs
│   ├── Review Records
│   ├── Comments / Notes
│   └── History
│
└── Right Inspector
    └── Metadata, source, events, attention level
Task buttons
Button	Action
New Task	Creates task
Start	Moves to In Progress
Mark Done	Completes task
Block	Marks Waiting/Blocked
Needs Review	Marks Needs Review
Archive	Archives task
Start Agent	Starts agent linked to task
Link Entity	Links object
Add Note	Creates linked note
Add Due Date	Sets due date
Create Calendar Event	Requires confirmation
Delete Task	Requires confirmation
16. Shared Calendar System

Calendar includes internal Zoid calendar data and Apple Calendar integration.

16.1 Sitemap
Calendar
├── Today
├── Week
├── Month
├── Content Calendar
├── Automation Schedule
└── Workspace Timeline
16.2 Calendar Layout
Calendar Page
│
├── Header
│   ├── View Switcher
│   ├── Date Navigation
│   ├── New Event Button
│   ├── Sync Apple Calendar Button
│   └── Calendar Settings Button
│
├── Calendar Body
│   ├── Events
│   ├── Task Due Dates
│   ├── Content Publishing Slots
│   ├── Automation Schedules
│   ├── Agent / Deploy Reminders
│   ├── Follow-ups
│   ├── Product Launch Dates
│   └── Business Deadlines
│
└── Right Inspector
    └── Selected event or calendar item
16.3 Calendar buttons
Button	Action
New Event	Creates Zoid event or Apple Calendar event after confirmation
Sync Apple Calendar	Reloads Apple Calendar events
Link to Task	Links event
Link to Note	Links event
Link to Client/Product	Links event
Check Conflict	Checks schedule conflict
Edit Event	Requires confirmation for external events
Delete Event	Requires confirmation
Open Source	Opens source calendar item if available
17. Shared History System

Every important action creates an Event, and history should be visible globally and per entity.

17.1 Sitemap
History
├── Global Recent History
├── Today Activity
├── Workspace History
└── Entity History
17.2 History Page Layout
History Page
│
├── Header
│   ├── Search Events
│   ├── Filter by Workspace
│   ├── Filter by Entity
│   ├── Filter by Actor
│   ├── Filter by Severity
│   └── Export Button
│
├── Event Timeline
│   ├── Timestamp
│   ├── Actor Type
│   ├── Workspace
│   ├── Event Type
│   ├── Summary
│   ├── Severity
│   └── Linked Entities
│
└── Right Inspector
    └── Selected event metadata and links
17.3 History buttons
Button	Action
Open Event	Opens event detail
Open Entity	Opens linked object
Filter	Opens filters
Export	Exports history selection
Copy Summary	Copies event summary
Create Task	Creates task from event
Link Entity	Adds link to event if permitted
18. Reviews System

Reviews are required for consequential actions and attach to entities through ReviewRecord.

18.1 Sitemap
Reviews
├── Pending Reviews
├── Approved
├── Required Fixes
├── Blocked
└── Review Record Detail
18.2 Reviews Page Layout
Reviews Page
│
├── Header
│   ├── Search Reviews
│   ├── Filter by Verdict
│   ├── Filter by Entity Type
│   ├── Filter by Workspace
│   └── New Review Button
│
├── Review List
│   ├── Reviewed Entity
│   ├── Reviewer Profile
│   ├── Verdict
│   ├── Evidence Summary
│   ├── Created At
│   └── Linked Events
│
└── Right Inspector
    └── Selected ReviewRecord
18.3 Review Record Detail
Review Record Detail
│
├── Header
│   ├── Reviewed Entity
│   ├── Verdict
│   ├── Reviewer Profile
│   ├── Created At
│   └── Open Entity Button
│
├── Evidence Summary
├── Required Fixes
├── Linked Events
├── Linked Tasks
└── Footer Actions
    ├── Approve and Continue
    ├── Create Fix Task
    ├── Re-run Review
    └── Archive Review
19. Settings
19.1 Sitemap
Settings
├── General
├── Workspaces
├── Widgets
├── Storage
├── CLI / Agent Profiles
├── Review Gate
├── Permissions
├── Integrations
│   ├── Gmail
│   ├── Apple Calendar
│   ├── GitHub
│   ├── Vercel
│   └── OmniSocials
├── Notifications
├── Security
├── Data / Indexing
├── Import / Migration
├── Export
└── About
19.2 Settings Layout
Settings
│
├── Settings Sidebar
│   ├── General
│   ├── Workspaces
│   ├── Widgets
│   ├── Storage
│   ├── CLI / Agent Profiles
│   ├── Review Gate
│   ├── Permissions
│   ├── Integrations
│   ├── Notifications
│   ├── Security
│   ├── Data / Indexing
│   ├── Import / Migration
│   ├── Export
│   └── About
│
├── Settings Content Area
│   └── Selected Settings Page
│
└── Optional Right Help / Status Panel
    ├── Current Setting Summary
    ├── Permission Impact
    └── Related Events
19.3 General Settings
General
│
├── Local Profile
├── App Startup Behavior
├── Default Workspace
├── Default Task Behavior
├── Default Date / Time Preferences
└── Buttons
    ├── Save Changes
    └── Reset General Settings
19.4 Workspaces Settings
Workspaces
│
├── Workspace List
│   ├── Today
│   ├── Code
│   ├── Agents
│   ├── Notes
│   ├── Content
│   ├── Automations
│   ├── Business
│   ├── Products
│   ├── Files
│   ├── Browser
│   └── Inbox
├── Workspace Visibility Controls
├── Default Workspace Order
└── Buttons
    ├── Save
    ├── Reset Order
    └── Restore Defaults
19.5 Widgets Settings
Widgets
│
├── Workspace Selector
├── Widget List for Selected Workspace
│   ├── Enabled Widgets
│   ├── Disabled Widgets
│   ├── Widget Size
│   └── Widget Order
└── Buttons
    ├── Add Widget
    ├── Remove Widget
    ├── Move Up
    ├── Move Down
    ├── Reset Workspace Widgets
    └── Save Layout
19.6 Storage Settings
Storage
│
├── Visible Folder Location
│   └── ~/Zoid/
├── App Support Folder Location
│   └── ~/Library/Application Support/Zoid/
├── Folder Health Checks
├── Database Status
├── Index Status
└── Buttons
    ├── Open Zoid Folder
    ├── Open App Support Folder
    ├── Rebuild Index
    ├── Check Storage
    └── Export Storage Report
19.7 CLI / Agent Profiles Settings
CLI / Agent Profiles
│
├── CLI Profiles List
├── Agent Profiles List
├── Default Reviewer Profile
├── Environment Variable References
├── Test Command Area
└── Buttons
    ├── New CLI Profile
    ├── New Agent Profile
    ├── Test Profile
    ├── Save
    ├── Disable
    └── Delete
19.8 Review Gate Settings
Review Gate
│
├── Consequential Action Rules
├── Reviewer Profile Selection
├── Module-Level Review Requirements
├── Evidence Requirements
├── Default Verdict Handling
└── Buttons
    ├── Save Rules
    ├── Test Review Flow
    └── Reset Review Defaults
19.9 Permissions Settings
Permissions
│
├── Module List
│   ├── Files
│   ├── Gmail
│   ├── Content / OmniSocials
│   ├── Code / Git / GitHub
│   ├── Deployments / Vercel
│   ├── Automations
│   ├── Calendar
│   ├── CLI / Agents
│   ├── Notes Import / Migration
│   └── Business / CRM
├── Policy Selector per Module
│   ├── Allowed Automatically
│   ├── Ask Before Action
│   ├── Always Blocked
│   ├── Require Reviewer Approval
│   └── Require Human Confirmation
└── Buttons
    ├── Save Policy
    ├── Reset Module
    └── Reset All Permissions
19.10 Integrations Settings
Integrations
│
├── Integration List
│   ├── Gmail
│   ├── Apple Calendar
│   ├── GitHub
│   ├── Vercel
│   └── OmniSocials
│
├── Selected Integration Detail
│   ├── Connection Status
│   ├── Account / Workspace Info
│   ├── Permissions
│   ├── Last Sync
│   ├── Error State
│   └── Logs
│
└── Buttons
    ├── Connect
    ├── Disconnect
    ├── Reconnect
    ├── Test Connection
    ├── Sync Now
    └── View Logs
19.11 Notifications Settings
Notifications
│
├── In-App Notification Rules
├── Native Notification Rules
├── Workspace Notification Levels
├── Severity Thresholds
├── Quiet / Suppression Rules
└── Buttons
    ├── Save
    ├── Send Test Notification
    └── Reset Defaults
19.12 Security Settings
Security
│
├── Keychain Status
├── Stored Credential References
├── Secret Redaction Status
├── Sensitive Data Handling
├── Permission Audit
└── Buttons
    ├── Check Keychain
    ├── Rotate Credential Reference
    ├── Clear Cached Tokens
    ├── Run Secret Scan
    └── Export Security Report
19.13 Data / Indexing Settings
Data / Indexing
│
├── SQLite Database Status
├── Notes Index Status
├── Search Index Status
├── Event Count / Health
├── Log Storage Status
└── Buttons
    ├── Rebuild Notes Index
    ├── Rebuild Search Index
    ├── Vacuum Database
    ├── Export Diagnostics
    └── Clear Cache
20. Entity Detail Layouts

These detail layouts should be reused wherever the entity appears.

20.1 Task Detail
Task Detail
├── Header
├── Description
├── Status / Priority / Due Date
├── Linked Entities
├── Agent Runs
├── Reviews
├── Notes
├── Events
└── Actions
20.2 Note Detail
Note Detail
├── Header
├── Metadata
├── Markdown Editor
├── Linked Entities
├── Backlinks
├── History
└── Actions
20.3 Repository Detail
Repository Detail
├── Header
├── Tabs
│   ├── Overview
│   ├── Git Status
│   ├── Diff
│   ├── Branches
│   ├── Commits
│   ├── PRs
│   ├── Deployments
│   ├── Launch Gate
│   ├── Linked Items
│   └── History
└── Inspector
20.4 Agent Run Detail
Agent Run Detail
├── Header
├── Summary
├── Prompt
├── Output
├── Raw Logs
├── Timeline
├── Linked Entities
├── Review
└── Events
20.5 Content Piece Detail
Content Piece Detail
├── Header
├── Pipeline Status
├── Brief
├── Draft
├── Assets
├── Captions
├── Platform Adaptations
├── Review
├── Publishing
├── Verification
└── History
20.6 Automation Detail
Automation Detail
├── Header
├── Overview
├── Schedule
├── Runs
├── Logs
├── Linked Entities
├── Failure Policy
└── History
20.7 Product Detail
Product Detail
├── Header
├── Overview
├── Roadmap
├── Tasks
├── Repos
├── Deployments
├── Releases
├── Notes / Decisions
├── Content
├── Automations
├── Clients / Companies
└── Timeline / History
20.8 Contact Detail
Contact Detail
├── Header
├── Overview
├── Emails
├── Tasks
├── Notes
├── Products
├── Follow-ups
└── History
20.9 Company Detail
Company Detail
├── Header
├── Overview
├── Contacts
├── Emails
├── Tasks
├── Notes
├── Products
├── Follow-ups
└── History
20.10 File Detail
File Detail
├── Header
├── Preview
├── Metadata
├── Linked Entities
├── Repo-Aware Status if applicable
├── Events
└── Actions
20.11 Browser Capture Detail
Browser Capture Detail
├── Header
├── URL / Source Page
├── Screenshot / Saved Page
├── Summary / Extracted Points
├── Linked Entities
├── Verification Evidence if applicable
└── History
21. Recommended Page Ordering in the App
21.1 Primary Sidebar Order
1. Today
2. Code
3. Agents
4. Notes
5. Content
6. Automations
7. Business
8. Products
9. Files
10. Browser
11. Inbox
21.2 Bottom Utility Order
1. Tasks
2. Calendar
3. History
4. Notifications
5. Settings
6. Local Profile
21.3 Common Detail Tab Order

For most entity detail pages:

1. Overview
2. Work-specific content
3. Linked entities
4. Reviews / approvals if applicable
5. Logs if applicable
6. History
21.4 Common Object Action Order

For most headers and inspectors:

1. Primary action
2. Secondary action
3. Link action
4. Review action
5. More menu
6. Destructive actions inside More menu only

Examples:

Task:
Start → Mark Done → Link Entity → Request Review → More

Content Piece:
Generate → Review → Schedule → Verify → More

Repo:
Run Checks → Start Agent → Open Launch Gate → Link Product → More

Automation:
Run Now → Pause/Resume → View Logs → Create Task → More
22. Global Empty States

Each workspace should have functional empty states.

Empty State
│
├── Explanation of what belongs here
├── Suggested first action
├── Optional secondary action
└── Optional link to settings/import/integration

Examples:

Workspace	Empty state primary button	Secondary button
Code	Scan for Repos	Add Repo Manually
Agents	Create Agent Profile	Configure CLI
Notes	New Note	Import Apple Notes
Content	New Content Piece	Create Content Plan
Automations	New Automation	Enable Default Automations
Business	New Contact	New Company
Products	New Product	Link Repo
Files	Open Zoid Folder	Add Folder
Browser	New Tab	Open Saved Pages
Inbox	Connect Gmail	View Zoid Notifications
23. Cross-Workspace Linking Structure

Every object should be linkable from every relevant workspace.

Entity Link Picker
│
├── Search Entities
├── Entity Type Filters
│   ├── Task
│   ├── Note
│   ├── Repository
│   ├── Agent Run
│   ├── Automation
│   ├── Content Piece
│   ├── Product
│   ├── Contact
│   ├── Company
│   ├── File
│   ├── Calendar Item
│   ├── Email Message
│   └── Browser Capture
├── Results List
├── Selected Entities
└── Footer Buttons
    ├── Cancel
    └── Link Selected

Common link buttons:

Button	Action
Link Entity	Opens entity picker
Unlink	Removes relationship
Open Linked Entity	Navigates to object
Create Linked Task	Creates task attached to object
Create Linked Note	Creates note attached to object
Attach File	Links file reference
Attach URL	Links browser page
24. Minimum Global Modals
24.1 New Task Modal
New Task Modal
├── Title
├── Description
├── Workspace
├── Status
├── Priority
├── Due Date
├── Tags
├── Linked Entities
└── Buttons
    ├── Cancel
    └── Create Task
24.2 New Note Modal
New Note Modal
├── Title
├── Workspace
├── Collection
├── Tags
├── Optional Linked Entity
└── Buttons
    ├── Cancel
    └── Create Note
24.3 Start Agent Run Modal
Start Agent Run Modal
├── Agent Profile
├── Working Directory
├── Prompt
├── Context Attachments
├── Permission Preview
├── Review Requirement Preview
└── Buttons
    ├── Cancel
    └── Start Run
24.4 Link Entity Modal
Link Entity Modal
├── Search
├── Filters
├── Results
├── Selected Entities
└── Buttons
    ├── Cancel
    └── Link Selected
24.5 Confirmation Modal
Confirmation Modal
├── Action Name
├── Reason Confirmation Is Required
├── Affected Objects
├── Risk / Consequence Summary
├── Optional Review Evidence
└── Buttons
    ├── Cancel
    └── Confirm
24.6 Review Modal
Review Modal
├── Reviewed Entity
├── Proposed Action
├── Evidence
├── Reviewer Profile
├── Review Result
└── Buttons
    ├── Cancel
    ├── Run Review
    ├── Apply Fixes
    └── Approve and Continue
25. Implementation-Oriented Route Naming

A practical route structure for the desktop app:

/today
/code
/code/discovery
/code/repos
/code/repos/:repoId
/code/repos/:repoId/status
/code/repos/:repoId/diff
/code/repos/:repoId/branches
/code/repos/:repoId/commits
/code/repos/:repoId/prs
/code/repos/:repoId/deployments
/code/repos/:repoId/launch-gate
/code/github
/code/vercel
/code/launch-gates

/agents
/agents/runs
/agents/runs/:runId
/agents/profiles
/agents/profiles/:profileId
/agents/reviewer
/agents/sessions
/agents/settings

/notes
/notes/all
/notes/:noteId
/notes/collections
/notes/collections/:collectionId
/notes/tags
/notes/tags/:tagId
/notes/imported/apple-notes
/notes/settings

/content
/content/calendar
/content/plans
/content/plans/:planId
/content/pieces
/content/pieces/:pieceId
/content/pieces/:pieceId/brief
/content/pieces/:pieceId/draft
/content/pieces/:pieceId/assets
/content/pieces/:pieceId/captions
/content/pieces/:pieceId/review
/content/pieces/:pieceId/publishing
/content/pieces/:pieceId/verification
/content/assets
/content/reviews
/content/omnisocials
/content/history
/content/failures
/content/settings

/automations
/automations/all
/automations/:automationId
/automations/:automationId/schedule
/automations/:automationId/runs
/automations/runs/:runId
/automations/failed
/automations/schedules
/automations/settings

/business
/business/contacts
/business/contacts/:contactId
/business/companies
/business/companies/:companyId
/business/follow-ups
/business/gmail
/business/tasks
/business/sops
/business/settings

/products
/products/all
/products/:productId
/products/:productId/roadmap
/products/:productId/tasks
/products/:productId/repos
/products/:productId/deployments
/products/:productId/releases
/products/:productId/notes
/products/:productId/content
/products/:productId/automations
/products/:productId/timeline
/products/roadmap
/products/launches
/products/decisions
/products/settings

/files
/files/explorer
/files/pinned
/files/zoid
/files/zoid/notes
/files/zoid/content
/files/zoid/assets
/files/zoid/imports
/files/zoid/exports
/files/repos
/files/assets
/files/captures
/files/linked
/files/settings

/browser
/browser/tabs
/browser/bookmarks
/browser/history
/browser/saved
/browser/captures
/browser/verification
/browser/settings

/inbox
/inbox/attention
/inbox/notifications
/inbox/agents
/inbox/automations
/inbox/reviews
/inbox/content
/inbox/reminders
/inbox/gmail
/inbox/gmail/thread/:threadId
/inbox/gmail/compose
/inbox/settings

/tasks
/tasks/:taskId
/calendar
/history
/reviews
/reviews/:reviewId
/permissions
/integrations
/settings
/settings/general
/settings/workspaces
/settings/widgets
/settings/storage
/settings/cli-agents
/settings/review-gate
/settings/permissions
/settings/integrations
/settings/notifications
/settings/security
/settings/data-indexing
/settings/import
/settings/export
/settings/about
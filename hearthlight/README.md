# Hearthlight

Hearthlight is a local-first household operating system for planning meals, tracking groceries, coordinating chores, and keeping a shared weekly rhythm for a family or house of roommates.

The goal is to build one cohesive product that helps a home run more smoothly without depending on a pile of disconnected apps, subscriptions, and reminder systems. Instead of treating grocery lists, calendars, pantry tracking, and routine planning as separate problems, Hearthlight would combine them into one shared workspace with practical automation and a calm interface.

## Product Vision

Hearthlight should feel like a reliable home dashboard rather than a productivity tool for work. A household opens it to answer questions like:

1. What are we eating this week?
2. What do we need to buy?
3. What chores are coming due?
4. What ingredients are already in the pantry?
5. What is the plan for the next few days?

The product should reduce mental overhead, especially for the person who usually becomes the default coordinator for everyone else.

## Core Problem

Household management is full of repeated micro-decisions:

1. Planning meals around real schedules.
2. Turning meal plans into shopping lists.
3. Avoiding duplicate grocery purchases.
4. Remembering recurring chores before they become urgent.
5. Keeping everyone aligned without constant texting.

Most existing tools solve only one slice of that workflow. Hearthlight is intended to connect those slices into a single system where each action naturally updates the next part of the household plan.

## Users

The primary users are:

1. Families managing meals, groceries, and recurring chores.
2. Couples trying to share household planning more evenly.
3. Roommates who need a lightweight shared system without excessive setup.

## Product Scope

Version one should focus on four integrated areas:

### 1. Weekly Planning

Users can create a weekly household plan with meals, events, and task load visible in one place. The system should make it easy to see busy nights, leftovers, and shopping deadlines.

### 2. Grocery and Pantry Management

Users can maintain a shared grocery list, pantry inventory, and staple items list. Planned meals should suggest needed ingredients, and pantry stock should reduce unnecessary purchases.

### 3. Chores and Recurring Routines

Users can define chores with recurrence, ownership, difficulty, and last-completed dates. The system should support fair rotation and show what is due soon.

### 4. Shared Household Dashboard

The home screen should answer the most important questions immediately:

1. Tonight's meal plan.
2. Items to buy.
3. Chores due today.
4. Upcoming schedule pressure points.
5. Low-stock pantry items.

## Guiding Principles

1. Local first: the app should work well for a single household without requiring cloud infrastructure on day one.
2. Calm by default: the interface should reduce stress, not create more notification noise.
3. Shared context over personal productivity: the product is for coordination, not individual task optimization.
4. Useful automation: automation should remove repeated work, but never make the household feel out of control.
5. Real-life flexibility: plans change, leftovers happen, and skipped chores should be easy to recover from.

## Major Features

### Household Setup

1. Create a household.
2. Add members.
3. Define dietary preferences, staple groceries, and default chores.
4. Set weekly planning cadence and shopping days.

### Meal Planning

1. Create a meal calendar.
2. Save reusable recipes.
3. Assign meals to days.
4. Mark meals as leftovers, quick meals, or hosted meals.
5. Generate ingredient needs from planned meals.

### Grocery Workflow

1. Add grocery items manually or from meal plans.
2. Group items by store section.
3. Mark pantry staples with restock thresholds.
4. Track purchased vs. unpurchased items.
5. Convert purchased goods into pantry inventory.

### Pantry Tracking

1. View current pantry inventory.
2. Record quantity, category, expiration estimate, and location.
3. Highlight low-stock and likely expiring items.
4. Suggest meals based on available ingredients.

### Chore Management

1. Create one-time and recurring chores.
2. Assign owners or rotate automatically.
3. Track completion history.
4. Balance workload across household members.
5. Surface overdue tasks without punishing UI.

### Dashboard and Reminders

1. Show today's household snapshot.
2. Provide upcoming reminders for meals, shopping, and chores.
3. Summarize the next three days in one view.
4. Keep the most actionable information visible with minimal navigation.

## Technical Plan

The project can be implemented in layered phases:

### Application Layer

A local web app with a responsive interface for desktop and mobile use. The primary experience should be optimized for a shared tablet in a kitchen as well as individual phones and laptops.

### Data Layer

A local database stores households, members, recipes, pantry inventory, shopping lists, chores, and weekly plans. The schema should be designed so syncing can be added later without reworking the core model.

### Logic Layer

Household logic connects domains together:

1. Meal plans create ingredient demand.
2. Pantry state offsets grocery demand.
3. Shopping completion updates pantry state.
4. Household schedule pressure influences meal suggestions.
5. Chore recurrence feeds daily dashboard summaries.

### Automation Layer

Helpful automations should remain transparent:

1. Build grocery suggestions from the weekly plan.
2. Flag low-stock staples.
3. Rotate chore ownership.
4. Suggest quick meals for busy evenings.
5. Generate a weekly planning checklist.

## Delivery Phases

### Phase 1: Foundation

Deliver a usable local app with:

1. Household creation.
2. Member profiles.
3. Weekly meal planning.
4. Shared grocery list.
5. Basic recurring chores.

Success in this phase means a household can use Hearthlight instead of notes, texts, and a separate grocery list app.

### Phase 2: Connected Workflows

Add the features that make the product feel integrated:

1. Recipe library.
2. Pantry inventory.
3. Grocery generation from meal plans.
4. Low-stock staple detection.
5. Dashboard summaries.

Success in this phase means one action in the system clearly improves the next part of the workflow.

### Phase 3: Smarter Household Support

Expand into decision support:

1. Meal suggestions based on schedule and pantry state.
2. Chore rotation balancing.
3. Planning prompts for the upcoming week.
4. Leftover-aware meal planning.
5. Expiration and waste reduction nudges.

Success in this phase means the app starts reducing planning effort, not just recording plans.

### Phase 4: Sync and Collaboration

Add broader multi-device collaboration:

1. Household sync.
2. Shared notifications.
3. Conflict handling for edits.
4. Offline-first reconciliation.
5. Optional household roles and permissions.

Success in this phase means the app feels dependable for a real multi-user household across devices.

## Risks and Open Questions

1. How much pantry tracking will users tolerate before it feels like bookkeeping?
2. What is the right balance between automation and manual control?
3. Which workflow should be truly excellent first: meals, groceries, or chores?
4. How should the product support both families and roommates without becoming generic?
5. What reminders are helpful versus annoying in a home context?

## First Milestone Build Plan

The first implementation milestone should deliver:

1. Household and member setup.
2. A week-view planner for meals.
3. A shared grocery list with categories and completion state.
4. A recurring chore list with due dates and completion tracking.
5. A simple dashboard that combines today's meals, shopping items, and chores.

This milestone is intentionally narrow: it should be good enough for one real household to try for two weeks and reveal where the product is genuinely helpful versus merely organized.

## Current Status

The first Hearthlight implementation slice is now underway on `codex/hearthlight-foundation`.

What is already working:

1. Hearthlight now uses AshDB and Kiln as libraries from inside `hearthlight/`
2. the app bootstraps a local AshDB file with tables for households, members, meals, groceries, and chores
3. the first seeded household dashboard renders server-side HTML through Kiln
4. the dashboard shows tonight's meal, household members, grocery items, and open chores
5. the first app actions support adding a grocery item and marking a chore complete
6. the first local asset path is wired for Hearthlight-specific styling
7. a long-running dashboard daemon entrypoint now exists for local browser testing and containerization
8. a weekly planner route now shows a full seven-day meal plan and supports saving meals into the plan
9. a pantry route now tracks on-hand inventory, flags low-stock staples, and accepts grocery purchases into pantry state
10. planner meals can now carry ingredient lists and generate grocery items directly into the shared shopping flow
11. a chores route now supports recurring chore creation, completion history, and completion that advances the next due state instead of just hiding work
12. a recipes route now stores reusable meals, scores pantry matches, and lets households drop saved recipes directly into the weekly plan
13. a setup route now lets a real household edit its name and weekly rhythm, and add members without editing seed data

The current goal is a real vertical slice, not a full product:

1. one seeded household
2. a dashboard plus planner route
3. a dashboard, planner, grocery, pantry, chores, recipe, and setup workflow with connected write actions
4. direct-compiler smoke coverage proving the app stack works end to end

## Local Development

The fastest local workflow is now:

1. `make -C hearthlight smoke`
2. `make -C hearthlight daemon`
3. `make -C hearthlight run PORT=9124 DB=/tmp/hearthlight.db`

Then browse to:

1. `http://127.0.0.1:9124/`
2. If an older local DB looks empty after a schema change, remove it and rerun with a fresh path such as `/tmp/hearthlight-v2.db`
3. direct-compiler smoke coverage proving the app stack works end to end

## Definition of Success

Hearthlight succeeds if it becomes the place a household checks when deciding what to cook, what to buy, and what needs doing, and if it makes that coordination feel lighter instead of more complicated.

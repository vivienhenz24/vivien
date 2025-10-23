# Content Management System

This project uses a markdown-based content management system for both essays and coding projects.

## Essays

Essays are stored as markdown files in `/src/content/essays/` with frontmatter metadata.

### Creating a New Essay

```bash
npm run create-essay
```

This will prompt you for:
- Essay title
- Date (YYYY-MM-DD format)
- Filename (without .md extension)

### Essay Frontmatter

Each essay file should start with frontmatter:

```markdown
---
title: "Your Essay Title"
date: "2025-01-15"
---

Your essay content here...
```

### Building Essays

Essays are automatically processed during the build process. The build script:
1. Reads all `.md` files from `/src/content/essays/`
2. Parses frontmatter and converts markdown to HTML
3. Saves processed data to `/src/lib/essays-data.json`

## Coding Projects

Projects are stored as markdown files in `/src/content/projects/` with frontmatter metadata.

### Creating a New Project

```bash
npm run create-project
```

This will prompt you for:
- Project title
- Project description
- Project URL
- Date (YYYY-MM-DD format)
- Filename (without .md extension)

### Project Frontmatter

Each project file should start with frontmatter:

```markdown
---
title: "Your Project Title"
description: "Brief description of your project"
url: "https://github.com/username/project"
date: "2025-01-15"
---

Your detailed project content here...
```

### Building Projects

Projects are automatically processed during the build process. The build script:
1. Reads all `.md` files from `/src/content/projects/`
2. Parses frontmatter and converts markdown to HTML
3. Saves processed data to `/src/lib/projects-data.json`

### Manual Project Build

To manually process projects:

```bash
npm run build-projects
```

## Content Structure

Both essays and projects are unified through the `/src/lib/content.ts` module, which provides:

- `getAllContent()` - Returns all content (essays + projects) sorted by date
- `getContentById(id)` - Returns specific content by ID
- `getAllContentIds()` - Returns all content IDs

## File Organization

```
src/
├── content/
│   ├── essays/           # Essay markdown files
│   └── projects/         # Project markdown files
├── lib/
│   ├── essays-data.json  # Processed essay data
│   ├── projects-data.json # Processed project data
│   ├── essays.ts         # Essay data access functions
│   ├── projects.ts       # Project data access functions
│   └── content.ts        # Unified content access
└── scripts/
    ├── build-essays.js   # Essay processing script
    ├── build-projects.js # Project processing script
    ├── create-essay.js   # Essay creation helper
    └── create-project.js # Project creation helper
```

## Development Workflow

1. Create new content using the helper scripts
2. Edit the markdown files with your content
3. Run the build process (happens automatically on `npm run build`)
4. Content is available through the unified API

## Tips

- Use descriptive filenames that match your project/essay titles
- Include proper frontmatter for all content
- The system automatically handles markdown-to-HTML conversion
- Content is sorted by date (newest first)
- Both essays and projects appear in the unified content feed

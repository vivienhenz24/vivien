# Vivien's Essay Website

A personal essay website built with Next.js where each essay is stored as a separate markdown file.

## Features

- **File-based essays**: Each essay is a separate markdown file in `src/content/essays/`
- **Automatic listing**: Essays are automatically listed on the homepage
- **Markdown support**: Write essays in markdown with frontmatter for metadata
- **Responsive design**: Clean, readable typography optimized for reading

## How to Add New Essays

### Method 1: Using the helper script (Recommended)

```bash
pnpm create-essay
```

This will prompt you for:
- Essay title
- Date (YYYY-MM-DD format)
- Filename (without .md extension)

### Method 2: Manual creation

1. Create a new `.md` file in `src/content/essays/`
2. Add frontmatter at the top:

```markdown
---
title: "Your Essay Title"
date: "2024-01-15"
---

Your essay content here...
```

## Essay File Structure

Each essay file should have:

1. **Frontmatter** (at the top, between `---` lines):
   - `title`: The essay title
   - `date`: Publication date in YYYY-MM-DD format

2. **Content**: Your essay in markdown format

Example:
```markdown
---
title: "My First Essay"
date: "2024-01-15"
---

This is my first essay. You can write in **bold**, *italic*, and use other markdown features.

## Subheadings

You can use different levels of headings:

### Level 3 heading

And so on...
```

## Development

```bash
# Install dependencies
pnpm install

# Start development server
pnpm dev

# Build for production
pnpm build

# Start production server
pnpm start
```

## File Structure

```
src/
├── app/
│   ├── [id]/
│   │   └── page.tsx          # Individual essay page
│   ├── globals.css           # Global styles
│   ├── layout.tsx            # Root layout
│   └── page.tsx              # Homepage with essay list
├── content/
│   └── essays/               # Essay markdown files
│       ├── thoughts.md
│       ├── reflections.md
│       └── creativity.md
└── lib/
    └── essays.ts             # Essay loading utilities
```

## Customization

- **Styling**: Edit `src/app/globals.css` for typography and layout
- **Layout**: Modify `src/app/layout.tsx` for site-wide changes
- **Essay format**: Update the frontmatter structure in `src/lib/essays.ts`

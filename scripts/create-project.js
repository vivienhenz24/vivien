#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const readline = require('readline');

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout
});

function question(prompt) {
  return new Promise((resolve) => {
    rl.question(prompt, resolve);
  });
}

async function createProject() {
  try {
    const title = await question('Project title: ');
    const description = await question('Project description: ');
    const url = await question('Project URL: ');
    const date = await question('Date (YYYY-MM-DD): ');
    const filename = await question('Filename (without .md): ');
    
    const projectsDir = path.join(process.cwd(), 'src/content/projects');
    const filePath = path.join(projectsDir, `${filename}.md`);
    
    const content = `---
title: "${title}"
description: "${description}"
url: "${url}"
date: "${date}"
---

Start writing your project content here...

## Overview

Describe what this project is about and why you built it.

## Technical Details

Explain the technical implementation, challenges, and solutions.

## What I Learned

Share insights and lessons learned from this project.

## Future Work

Describe any planned improvements or extensions.

`;
    
    fs.writeFileSync(filePath, content);
    console.log(`\n✅ Project created: ${filePath}`);
    console.log(`\nYou can now edit the file and add your content.`);
    console.log(`\nRun 'npm run build-projects' to process the new project.`);
    
  } catch (error) {
    console.error('Error creating project:', error);
  } finally {
    rl.close();
  }
}

createProject();

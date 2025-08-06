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

async function createEssay() {
  try {
    const title = await question('Essay title: ');
    const date = await question('Date (YYYY-MM-DD): ');
    const filename = await question('Filename (without .md): ');
    
    const essaysDir = path.join(process.cwd(), 'src/content/essays');
    const filePath = path.join(essaysDir, `${filename}.md`);
    
    const content = `---
title: "${title}"
date: "${date}"
---

Start writing your essay here...

`;
    
    fs.writeFileSync(filePath, content);
    console.log(`\n✅ Essay created: ${filePath}`);
    console.log(`\nYou can now edit the file and add your content.`);
    
  } catch (error) {
    console.error('Error creating essay:', error);
  } finally {
    rl.close();
  }
}

createEssay(); 
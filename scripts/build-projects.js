const fs = require('fs')
const path = require('path')
const matter = require('gray-matter')

const projectsDirectory = path.join(process.cwd(), 'src/content/projects')
const outputPath = path.join(process.cwd(), 'src/lib/projects-data.json')

async function processProjects() {
  console.log('Processing projects at build time...')
  
  // Dynamic imports for ES modules
  const { remark } = await import('remark')
  const remarkHtml = await import('remark-html')
  const remarkGfm = await import('remark-gfm')
  
  const fileNames = fs.readdirSync(projectsDirectory)
  const projects = fileNames.map((fileName) => {
    const id = fileName.replace(/\.md$/, '')
    const fullPath = path.join(projectsDirectory, fileName)
    const fileContents = fs.readFileSync(fullPath, 'utf8')
    
    // Parse frontmatter
    const matterResult = matter(fileContents)
    
    // Convert markdown to HTML
    const processedContent = remark()
      .use(remarkGfm.default)
      .use(remarkHtml.default)
      .processSync(matterResult.content)
    const contentHtml = processedContent.toString()
    
    // Parse and format date
    const originalDate = new Date(matterResult.data.date + 'T00:00:00') // Add time to avoid timezone issues
    const formattedDate = originalDate.toLocaleDateString('en-US', { 
      year: 'numeric', 
      month: 'long' 
    })
    
    return {
      id,
      title: matterResult.data.title,
      description: matterResult.data.description,
      url: matterResult.data.url,
      date: formattedDate,
      content: matterResult.content,
      htmlContent: contentHtml,
      originalDate: originalDate.toISOString(), // Store as ISO string for JSON
    }
  })
  
  // Sort by date
  const sortedProjects = projects.sort((a, b) => {
    return new Date(b.originalDate).getTime() - new Date(a.originalDate).getTime()
  })
  
  // Write to JSON file
  fs.writeFileSync(outputPath, JSON.stringify(sortedProjects, null, 2))
  console.log(`Processed ${sortedProjects.length} projects and saved to ${outputPath}`)
}

processProjects().catch(console.error)

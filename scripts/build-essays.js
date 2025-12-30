const fs = require('fs')
const path = require('path')
const matter = require('gray-matter')

const essaysDirectory = path.join(process.cwd(), 'src/content/essays')
const outputPath = path.join(process.cwd(), 'src/lib/essays-data.json')

async function processEssays() {
  console.log('Processing essays at build time...')

  // Dynamic imports for ES modules
  const { remark } = await import('remark')
  const remarkHtml = await import('remark-html')
  const remarkGfm = await import('remark-gfm')

  const fileNames = fs.readdirSync(essaysDirectory)
  const essays = fileNames.map((fileName) => {
    const id = fileName.replace(/\.md$/, '')
    const fullPath = path.join(essaysDirectory, fileName)
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
    const originalDate = new Date(matterResult.data.date)
    const formattedDate = originalDate.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long'
    })

    return {
      id,
      title: matterResult.data.title,
      date: formattedDate,
      content: matterResult.content,
      htmlContent: contentHtml,
      originalDate: originalDate.toISOString(), // Store as ISO string for JSON
    }
  })

  // Filter and sort
  const filteredEssays = essays.filter(essay =>
    essay.id !== 'orwell' && essay.id !== 'betweenthelines' && essay.id !== 'denial'
  )

  const sortedEssays = filteredEssays.sort((a, b) => {
    return new Date(b.originalDate).getTime() - new Date(a.originalDate).getTime()
  })

  // Write to JSON file
  fs.writeFileSync(outputPath, JSON.stringify(sortedEssays, null, 2))
  console.log(`Processed ${sortedEssays.length} essays and saved to ${outputPath}`)
}

processEssays().catch(console.error)

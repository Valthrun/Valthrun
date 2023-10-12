# Contributing to the Wiki
Contributing to the Wiki is a straightforward process that allows individuals to expand and enhance the knowledge base collaboratively. 
The Wiki is built using Docsify and is stored in the Valthrun repository.  
All the content within the Wiki can be located in the docs folder,
and the content files are typically written in Markdown for ease of editing.

## Adding New Pages
To contribute by adding new pages to the Wiki, follow these steps:

1. Clone the Valthrun repository to your local machine.
2. In the `docs` folder, create a new Markdown file (`.md`) with the appropriate content for your new page.
3. To ensure that your new page is accessible from the sidebar, you must also edit the `_sidebar.md` file and include a link to your newly created page.
  

## Previewing Changes
To preview any changes you make to the Wiki pages before finalizing them, you can follow these steps:

1. Install Docsify on your local machine. You can install Docsify using npm (Node Package Manager).  
If you don't have npm installed, you can get it [here](https://www.npmjs.com/).
```bash
npm i docsify-cli -g
```

2. Once Docsify is installed, navigate to the Valthrun repository, where the `docs` folder is located.
3. Run the following command to serve the Wiki locally:
```
docsify serve docs
```
4. Docsify will provide you with a URL that you can open in your web browser.   
Any changes you make to the content files will be immediately reflected on the live preview page, 
allowing you to see how your edits will appear to users.

This live preview feature provides a convenient way to review your
contributions and ensure that they are accurate and well-formatted before publishing them.
  

## More about docsify
For more information about Docsify and its capabilities, you can refer to the official Docsify documentation: [Docsify Documentation](https://docsify.js.org/#/).  
This resource offers detailed information and tips on using Docsify effectively for
building and maintaining Wikis and other documentation projects.

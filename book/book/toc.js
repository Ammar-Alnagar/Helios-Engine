// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="introduction.html">Introduction</a></li><li class="chapter-item expanded affix "><li class="part-title">Getting Started</li><li class="chapter-item expanded "><a href="getting_started/installation.html"><strong aria-hidden="true">1.</strong> Installation</a></li><li class="chapter-item expanded "><a href="getting_started/configuration.html"><strong aria-hidden="true">2.</strong> Configuration</a></li><li class="chapter-item expanded "><a href="getting_started/your_first_agent.html"><strong aria-hidden="true">3.</strong> Your First Agent</a></li><li class="chapter-item expanded affix "><li class="part-title">Core Concepts</li><li class="chapter-item expanded "><a href="core_concepts/agents.html"><strong aria-hidden="true">4.</strong> Agents</a></li><li class="chapter-item expanded "><a href="core_concepts/react.html"><strong aria-hidden="true">5.</strong> ReAct (Reasoning and Acting)</a></li><li class="chapter-item expanded "><a href="core_concepts/llms.html"><strong aria-hidden="true">6.</strong> LLMs</a></li><li class="chapter-item expanded "><a href="core_concepts/chat.html"><strong aria-hidden="true">7.</strong> Chat</a></li><li class="chapter-item expanded "><a href="core_concepts/error_handling.html"><strong aria-hidden="true">8.</strong> Error Handling</a></li><li class="chapter-item expanded affix "><li class="part-title">Tools</li><li class="chapter-item expanded "><a href="tools/using_tools.html"><strong aria-hidden="true">9.</strong> Using Tools</a></li><li class="chapter-item expanded "><a href="tools/creating_tools.html"><strong aria-hidden="true">10.</strong> Creating Tools</a></li><li class="chapter-item expanded "><a href="tools/tool_builder.html"><strong aria-hidden="true">11.</strong> Tool Builder</a></li><li class="chapter-item expanded affix "><li class="part-title">Forest of Agents</li><li class="chapter-item expanded "><a href="forest_of_agents/introduction.html"><strong aria-hidden="true">12.</strong> Introduction</a></li><li class="chapter-item expanded "><a href="forest_of_agents/communication.html"><strong aria-hidden="true">13.</strong> Communication</a></li><li class="chapter-item expanded "><a href="forest_of_agents/coordinator_based_planning.html"><strong aria-hidden="true">14.</strong> Coordinator-Based Planning</a></li><li class="chapter-item expanded affix "><li class="part-title">Retrieval-Augmented Generation (RAG)</li><li class="chapter-item expanded "><a href="rag/introduction.html"><strong aria-hidden="true">15.</strong> Introduction</a></li><li class="chapter-item expanded "><a href="rag/vector_stores.html"><strong aria-hidden="true">16.</strong> Vector Stores</a></li><li class="chapter-item expanded "><a href="rag/embeddings.html"><strong aria-hidden="true">17.</strong> Embeddings</a></li><li class="chapter-item expanded affix "><li class="part-title">API Reference</li><li class="chapter-item expanded "><a href="api_reference/serve_module.html"><strong aria-hidden="true">18.</strong> Serve Module</a></li><li class="chapter-item expanded affix "><li class="part-title">Examples</li><li class="chapter-item expanded "><a href="examples/overview.html"><strong aria-hidden="true">19.</strong> Overview</a></li><li class="chapter-item expanded affix "><li class="part-title">Contributing</li><li class="chapter-item expanded "><a href="contributing/how_to_contribute.html"><strong aria-hidden="true">20.</strong> How to Contribute</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);

(function() {
    function getCytoscapeConfig(elements) {
        return {
            container: document.getElementById('cytoscape-graph'),
            elements: elements,
            style: [
                {
                    selector: 'node',
                    style: {
                        'background-color': '#666',
                        'label': 'data(label)',
                        'width': 30,
                        'height': 30
                    }
                },
                {
                    selector: 'edge',
                    style: {
                        'width': 2,
                        'line-color': '#ccc',
                        'target-arrow-color': '#ccc',
                        'target-arrow-shape': 'triangle',
                        'curve-style': 'bezier',
                        'label': 'data(label)'
                    }
                }
            ],
            layout: {
                name: 'cose',
                fit: true,
                padding: 30
            }
        };
    }
    
    function initGraph() {
        // Remove existing instance if any
        const existing = document.getElementById('cytoscape-graph');
        if (existing && window.cyInstance) {
            window.cyInstance.destroy();
        }
        
        // Get container
        let container = document.getElementById('cytoscape-graph');
        if (!container) {
            console.error('Cytoscape container not found');
            return;
        }
        
        // Initialize Cytoscape with a simple toy graph
        const elements = [
            // Nodes
            { data: { id: 'a', label: 'Cat' } },
            { data: { id: 'b', label: 'Mammal' } },
            { data: { id: 'c', label: 'Animal' } },
            { data: { id: 'd', label: 'Dog' } },
            { data: { id: 'e', label: 'Pet' } },
            // Edges
            { data: { id: 'ab', source: 'a', target: 'b', label: 'is_a' } },
            { data: { id: 'bc', source: 'b', target: 'c', label: 'is_a' } },
            { data: { id: 'db', source: 'd', target: 'b', label: 'is_a' } },
            { data: { id: 'ae', source: 'a', target: 'e', label: 'is_a' } },
            { data: { id: 'de', source: 'd', target: 'e', label: 'is_a' } }
        ];
        
        window.cyInstance = cytoscape(getCytoscapeConfig(elements));
    }
    
    // Clear graph function
    window.clearGraph = function() {
        if (window.cyInstance) {
            window.cyInstance.elements().remove();
        }
    };
    
    // Create random graph function
    window.createRandomGraph = function() {
        if (!window.cyInstance) {
            initGraph();
            return;
        }
        
        // Clear existing graph
        window.cyInstance.elements().remove();
        
        // Generate random graph
        const nodeCount = Math.floor(Math.random() * 8) + 3; // 3-10 nodes
        const nodes = [];
        const edges = [];
        const nodeIds = [];
        
        // Create nodes
        for (let i = 0; i < nodeCount; i++) {
            const id = 'n' + i;
            nodeIds.push(id);
            nodes.push({
                data: {
                    id: id,
                    label: 'Node ' + i
                }
            });
        }
        
        // Create random edges (connect each node to 1-3 other nodes)
        for (let i = 0; i < nodeCount; i++) {
            const edgeCount = Math.floor(Math.random() * 3) + 1; // 1-3 edges per node
            const connected = new Set();
            
            for (let j = 0; j < edgeCount; j++) {
                let targetIdx;
                do {
                    targetIdx = Math.floor(Math.random() * nodeCount);
                } while (targetIdx === i || connected.has(targetIdx));
                
                connected.add(targetIdx);
                edges.push({
                    data: {
                        id: 'e' + i + '_' + targetIdx,
                        source: nodeIds[i],
                        target: nodeIds[targetIdx],
                        label: '→'
                    }
                });
            }
        }
        
        // Add nodes and edges
        window.cyInstance.add(nodes);
        window.cyInstance.add(edges);
        
        // Re-run layout
        window.cyInstance.layout({
            name: 'cose',
            fit: true,
            padding: 30
        }).run();
    };
    
    // Wait for Cytoscape to be available
    function waitForCytoscape() {
        if (typeof cytoscape !== 'undefined') {
            // Wait a bit more to ensure DOM is ready
            if (document.getElementById('cytoscape-graph')) {
                initGraph();
            } else {
                setTimeout(waitForCytoscape, 50);
            }
        } else {
            setTimeout(waitForCytoscape, 50);
        }
    }
    
    // Start waiting for Cytoscape
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', waitForCytoscape);
    } else {
        waitForCytoscape();
    }
    
    // Set up button handlers
    function setupButtons() {
        const clearBtn = document.getElementById('clear-graph-btn');
        const randomBtn = document.getElementById('random-graph-btn');
        
        if (clearBtn) {
            clearBtn.addEventListener('click', function() {
                if (window.clearGraph) {
                    window.clearGraph();
                }
            });
        }
        
        if (randomBtn) {
            randomBtn.addEventListener('click', function() {
                if (window.createRandomGraph) {
                    window.createRandomGraph();
                }
            });
        }
    }
    
    // Setup buttons when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', setupButtons);
    } else {
        setupButtons();
    }
})();

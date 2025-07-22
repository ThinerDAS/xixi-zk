document.addEventListener('DOMContentLoaded', async function() {
    // Initialize loader and load data
    const loader = new MotadataLoader();
    try {
        await loader.loadFromJson('game_data.json');
        initVisualization(loader);
        initDataTables(loader);
        initTabs();
    } catch (error) {
        console.error('Initialization failed:', error);
        alert('Failed to load game data. Please check console for details.');
    }
});

function initVisualization(loader) {
    const canvas = document.getElementById('map-canvas');
    const layerSelect = document.getElementById('layer-select');
    const nodeInfoPanel = document.getElementById('node-info');
    
    // Populate layer select
    const layers = loader.getLayers();
    layers.forEach(z => {
        const option = document.createElement('option');
        option.value = z;
        option.textContent = `Floor ${z}`;
        layerSelect.appendChild(option);
    });
    
    // Draw initial layer
    if (layers.length > 0) {
        drawLayer(loader, canvas, nodeInfoPanel, layers[0]);
    }
    
    // Handle layer changes
    layerSelect.addEventListener('change', () => {
        const z = parseInt(layerSelect.value);
        drawLayer(loader, canvas, nodeInfoPanel, z);
    });
}

function drawLayer(loader, canvas, infoPanel, z) {
    // Clear canvas
    canvas.innerHTML = '';
    
    const nodes = loader.getNodesByLayer(z);
    if (nodes.length === 0) return;
    
    // Shared coordinate calculation parameters with expanded padding
    const canvasParams = {
        width: canvas.offsetWidth,
        height: canvas.offsetHeight,
        padding: 80,  // Increased from 40 to 80
        minX: Infinity,
        maxX: -Infinity,
        minY: Infinity,
        maxY: -Infinity,
        boundaryOffset: 60  // Additional safety margin
    };

    // Calculate min/max coordinates
    nodes.forEach(node => {
        canvasParams.minX = Math.min(canvasParams.minX, node[0]);
        canvasParams.maxX = Math.max(canvasParams.maxX, node[0]);
        canvasParams.minY = Math.min(canvasParams.minY, node[1]);
        canvasParams.maxY = Math.max(canvasParams.maxY, node[1]);
    });

    // Calculate scaling factors
    const rangeX = Math.max(canvasParams.maxX - canvasParams.minX, 10);
    const rangeY = Math.max(canvasParams.maxY - canvasParams.minY, 10);
    canvasParams.scale = Math.min(
        (canvasParams.width - 2 * canvasParams.padding) / rangeX,
        (canvasParams.height - 2 * canvasParams.padding) / rangeY
    );

    // Unified position calculation function
    function calculatePosition(x, y) {
        // Snap to nearest grid point (40px spacing)
        const gridSize = 40;
        const rawX = canvasParams.padding + (x - canvasParams.minX) * canvasParams.scale;
        const rawY = canvasParams.padding + (y - canvasParams.minY) * canvasParams.scale;
        
        return {
            x: Math.round(rawX / gridSize) * gridSize,
            y: Math.round(rawY / gridSize) * gridSize
        };
    }

    // Draw boundary lines first (under nodes)
    const createBoundaryLine = (type, start, end) => {
        const line = document.createElement('div');
        line.className = 'boundary-line';
        
        const offset = 40; // 1 tile size
        const extend = 40; // additional extension
        
        if (type === 'horizontal') {
            // Top/bottom lines
            const yOffset = (start.y === minPos.y) ? -offset : offset;
            line.style.top = `${start.y + yOffset}px`;
            line.style.left = `${start.x - extend}px`;
            line.style.width = `${end.x - start.x + 2*extend}px`;
            line.style.height = '2px';
        } else {
            // Left/right lines offset horizontally and extended vertically
            const xOffset = (start.x === minPos.x) ? -offset : offset;
            line.style.top = `${start.y - extend}px`;
            line.style.left = `${start.x + xOffset}px`;
            line.style.width = '2px';
            line.style.height = `${end.y - start.y + 2*extend}px`;
        }

        console.log(`Drawing ${type} boundary from`,
                  `(${line.style.left},${line.style.top})`,
                  `to (${type==='horizontal'?parseInt(line.style.left)+parseInt(line.style.width):line.style.left},`,
                  `${type==='vertical'?parseInt(line.style.top)+parseInt(line.style.height):line.style.top})`);
        return line;
    };

    const minPos = calculatePosition(canvasParams.minX, canvasParams.minY);
    const maxPos = calculatePosition(canvasParams.maxX, canvasParams.maxY);

    // Verify boundary positions
    console.log('Boundary positions:', {
        minPos, maxPos,
        topLeft: {x: minPos.x - 40, y: minPos.y - 40},
        bottomRight: {x: maxPos.x + 40, y: maxPos.y + 40}
    });

    // Draw all boundaries with offset and extension
    const boundaries = [
        createBoundaryLine('horizontal',
                         {x: minPos.x, y: minPos.y},
                         {x: maxPos.x, y: minPos.y}), // top
        createBoundaryLine('horizontal',
                         {x: minPos.x, y: maxPos.y},
                         {x: maxPos.x, y: maxPos.y}), // bottom
        createBoundaryLine('vertical',
                         {x: minPos.x, y: minPos.y},
                         {x: minPos.x, y: maxPos.y}), // left
        createBoundaryLine('vertical',
                         {x: maxPos.x, y: minPos.y},
                         {x: maxPos.x, y: maxPos.y}) // right
    ];

    // Verify all boundaries were created
    console.log('Created boundaries:', boundaries);

    boundaries.forEach(line => canvas.appendChild(line));
    
    // Enhanced node position planner
    const positionPlanner = new Map();
    const drawnMinors = new Set();

    // Pre-plan all minor node positions
    nodes.forEach(node => {
      const minorIndices = loader.majorMinorAdj[node.index];
      minorIndices.forEach(minorIndex => {
        if (drawnMinors.has(minorIndex)) return;
        
        const minorInfo = loader.getMinorNodeInfo(minorIndex);
        const connectedMajors = minorInfo.adjMajors.filter(majorIdx => {
          return loader.majorCoords[majorIdx][2] === z;
        });

        // Calculate ideal position (average of connected majors)
        let idealPos = {x: 0, y: 0};
        connectedMajors.forEach(majorIdx => {
          const majorPos = calculatePosition(...loader.majorCoords[majorIdx]);
          idealPos.x += majorPos.x;
          idealPos.y += majorPos.y;
        });
        idealPos.x /= connectedMajors.length;
        idealPos.y /= connectedMajors.length;
        
        positionPlanner.set(minorIndex, {
          ideal: idealPos,
          connectedMajors,
          adjusted: null
        });
      });
    });

    // Smart position adjustment with boundary constraints
    positionPlanner.forEach((plan, minorIndex) => {
      if (drawnMinors.has(minorIndex)) return;
      
      const {ideal, connectedMajors} = plan;
      let bestPosition = ideal;
      let bestScore = -Infinity;
      
      // Test 16 positions around ideal point with boundary checks
      for (let angle = 0; angle < Math.PI * 2; angle += Math.PI / 8) {
        let testPos = {
          x: ideal.x + Math.cos(angle) * 60,
          y: ideal.y + Math.sin(angle) * 60
        };
        
        // Enforce boundary constraints
        testPos.x = Math.max(
          canvasParams.padding - canvasParams.boundaryOffset,
          Math.min(
            canvasParams.width - canvasParams.padding + canvasParams.boundaryOffset,
            testPos.x
          )
        );
        testPos.y = Math.max(
          canvasParams.padding - canvasParams.boundaryOffset,
          Math.min(
            canvasParams.height - canvasParams.padding + canvasParams.boundaryOffset,
            testPos.y
          )
        );
        
        // Collision detection (45px safe distance)
        let collision = false;
        positionPlanner.forEach((otherPlan, otherIndex) => {
          if (otherIndex === minorIndex || !otherPlan.adjusted) return;
          const dist = Math.sqrt(
            Math.pow(testPos.x - otherPlan.adjusted.x, 2) +
            Math.pow(testPos.y - otherPlan.adjusted.y, 2)
          );
          if (dist < 45) collision = true;
        });
        
        // Connection distance score
        let distanceScore = 0;
        connectedMajors.forEach(majorIdx => {
          const majorPos = calculatePosition(...loader.majorCoords[majorIdx]);
          distanceScore += 100 - Math.sqrt(
            Math.pow(testPos.x - majorPos.x, 2) +
            Math.pow(testPos.y - majorPos.y, 2)
          );
        });
        
        const finalScore = distanceScore - (collision ? 1000 : 0);
        if (finalScore > bestScore) {
          bestScore = finalScore;
          bestPosition = testPos;
        }
      }
      
      plan.adjusted = bestPosition;
    });
    
    // First pass: draw major nodes and connections
    nodes.forEach(node => {
        const pos = calculatePosition(node[0], node[1]);
        
        // Draw connections first (so they appear under nodes)
        const adjNodes = loader.majorAdj[node.index];
        adjNodes.forEach(adjIndex => {
            const adjNode = loader.majorCoords[adjIndex];
            if (adjNode[2] === z) { // Only draw connections on same layer
                const adjPos = calculatePosition(adjNode[0], adjNode[1]);
                
                const line = document.createElement('div');
                line.className = 'edge';
                line.style.left = `${pos.x}px`;
                line.style.top = `${pos.y}px`;
                line.style.width = `${Math.sqrt(Math.pow(adjPos.x - pos.x, 2) + Math.pow(adjPos.y - pos.y, 2))}px`;
                line.style.transform = `rotate(${Math.atan2(adjPos.y - pos.y, adjPos.x - pos.x)}rad)`;
                line.style.zIndex = '0'; // Ensure edges are below nodes
                canvas.appendChild(line);
            }
        });

        // Draw major node with type-prefixed identifier
        const nodeElement = document.createElement('div');
        nodeElement.className = 'node major-node';
        nodeElement.style.left = `${pos.x - 12}px`;
        nodeElement.style.top = `${pos.y - 12}px`;
        nodeElement.dataset.index = `major-${node.index}`; // Add type prefix
        nodeElement.dataset.originalIndex = node.index; // Store original index
        nodeElement.style.zIndex = '10'; // Ensure nodes are above edges
        
        // Add index label
        const label = document.createElement('div');
        label.className = 'node-label';
        label.textContent = node.index;
        label.style.position = 'absolute';
        label.style.left = '50%';
        label.style.top = '50%';
        label.style.transform = 'translate(-50%, -50%)';
        label.style.color = 'white';
        label.style.fontSize = '10px';
        label.style.fontWeight = 'bold';
        nodeElement.appendChild(label);
        
        // Define highlight functions outside to avoid redefinition
        const highlightMajorNode = (element, index) => {
            console.log('Highlighting major node:', index,
                      'major adj:', JSON.stringify(loader.majorAdj[index]),
                      'minor adj:', JSON.stringify(loader.majorMinorAdj[index]));
            
            // Clear any existing highlights first
            removeAllHighlights();
            
            // Highlight this major node
            element.classList.add('highlight');
            
            // 1. Highlight DIRECTLY connected MAJOR nodes
            loader.majorAdj[index].forEach(majorIdx => {
                const majorElement = document.querySelector(`.major-node[data-index="major-${majorIdx}"]`);
                if (majorElement) {
                    console.log('Highlighting directly connected major:', majorIdx);
                    majorElement.classList.add('highlight-adjacent');
                }
            });
            
            // 2. Highlight connected MINOR nodes
            const minorIndices = loader.majorMinorAdj[index] || [];
            minorIndices.forEach(minorIdx => {
                const minorElement = document.querySelector(`.minor-node[data-index="${minorIdx}"]`);
                if (minorElement) {
                    console.log('Highlighting connected minor:', minorIdx);
                    minorElement.classList.add('highlight-minor');
                }
            });
            
            // Debug: Verify highlighted elements
            setTimeout(() => {
                console.log('Currently highlighted elements:',
                    Array.from(document.querySelectorAll('.highlight, .highlight-adjacent, .highlight-minor'))
                        .map(el => el.dataset.index)
                );
            }, 100);
        };
        
        const removeAllHighlights = () => {
            document.querySelectorAll('.highlight, .highlight-adjacent, .highlight-minor').forEach(el => {
                el.classList.remove('highlight', 'highlight-adjacent', 'highlight-minor');
            });
        };
        
        // Add hover handlers for major nodes
        nodeElement.addEventListener('mouseover', function() {
            const index = node.index;
            highlightMajorNode(this, index);
            
            // Show node info (without duplicates)
            const info = loader.getMajorNodeInfo(node.index);
            let html = `<h4>Major Node ${node.index}</h4>`;
            html += `<p><strong>Position:</strong> (${info.coords[0]}, ${info.coords[1]}, ${info.coords[2]})</p>`;
            
            if (info.desc.Enemy !== undefined) {
                const enemy = loader.enemyData[info.desc.Enemy];
                html += `<p><strong>Type:</strong> Enemy (ID: ${info.desc.Enemy})</p>`;
                html += `<p><strong>Stats:</strong> HP:${enemy.hp} ATK:${enemy.atk} DEF:${enemy.def}</p>`;
                html += `<p><strong>EXP:</strong> ${enemy.exp} AT Times:${enemy.attimes}</p>`;
                if (enemy.magic || enemy.solid || enemy.speedy || enemy.nobomb) {
                    html += `<p><strong>Special:</strong> `;
                    const specials = [];
                    if (enemy.magic) specials.push('Magic');
                    if (enemy.solid) specials.push('Solid');
                    if (enemy.speedy) specials.push('Speedy');
                    if (enemy.nobomb) specials.push('NoBomb');
                    html += specials.join(', ') + `</p>`;
                }
            } else if (info.desc.Delta) {
                html += `<p><strong>Type:</strong> Delta Effect</p>`;
                html += `<ul>`;
                info.desc.Delta.forEach(([attr, val]) => {
                    html += `<li>${attr}: ${val > 0 ? '+' : ''}${val}</li>`;
                });
                html += `</ul>`;
            }
            
            html += `<p><strong>Adjacent Majors:</strong><br>`;
            info.adjMajors.forEach(adjIndex => {
                const adjCoord = loader.majorCoords[adjIndex];
                html += `• ${adjIndex} (${adjCoord[0]},${adjCoord[1]},${adjCoord[2]})<br>`;
            });
            html += `</p>`;
            html += `<p><strong>Connected Minors:</strong> ${info.adjMinors.join(', ')}</p>`;
            
            infoPanel.innerHTML = html;
        });
        
        nodeElement.addEventListener('mouseout', removeAllHighlights);
        
        canvas.appendChild(nodeElement);
    });

    // Second pass: draw minor nodes after all major nodes to avoid overlap
    const minorPositions = new Map(); // Track minor node positions
    
    nodes.forEach(node => {
        const pos = calculatePosition(node[0], node[1]);
        const minorIndices = loader.majorMinorAdj[node.index];
        
        minorIndices.forEach(minorIndex => {
            if (drawnMinors.has(minorIndex)) return;
            
            const minorInfo = loader.getMinorNodeInfo(minorIndex);
            const showOnLayer = minorInfo.adjMajors.some(majorIdx => {
                const majorZ = loader.majorCoords[majorIdx][2];
                return majorZ === z;
            });
            
            if (showOnLayer) {
                // Use pre-planned position from positionPlanner
                const plan = positionPlanner.get(minorIndex);
                let minorPos = plan?.adjusted || {
                    x: Math.max(
                        canvasParams.padding - canvasParams.boundaryOffset,
                        Math.min(
                            canvasParams.width - canvasParams.padding + canvasParams.boundaryOffset,
                            pos.x + 40 + Math.random() * 20
                        )
                    ),
                    y: Math.max(
                        canvasParams.padding - canvasParams.boundaryOffset,
                        Math.min(
                            canvasParams.height - canvasParams.padding + canvasParams.boundaryOffset,
                            pos.y + 40 + Math.random() * 20
                        )
                    )
                };
                
                // Final collision check with existing minors
                let hasCollision = false;
                minorPositions.forEach((existingPos, existingMinor) => {
                    const dist = Math.sqrt(
                        Math.pow(minorPos.x - existingPos.x, 2) +
                        Math.pow(minorPos.y - existingPos.y, 2)
                    );
                    if (dist < 45) hasCollision = true;
                });
                
                // If collision found, adjust position slightly
                if (hasCollision) {
                    const angle = Math.random() * Math.PI * 2;
                    minorPos = {
                        x: minorPos.x + Math.cos(angle) * 10,
                        y: minorPos.y + Math.sin(angle) * 10
                    };
                }
                
                // Enhanced position calculation with collision avoidance
                let avgX = 0;
                let avgY = 0;
                let count = 0;
                const MIN_DISTANCE = 60; // Increased minimum distance
                const AVOIDANCE_RADIUS = 80; // Area to check for other majors
                
                minorInfo.adjMajors.forEach(majorIdx => {
                    const major = loader.majorCoords[majorIdx];
                    if (major[2] === z) {
                        const majorPos = calculatePosition(major[0], major[1]);
                        avgX += majorPos.x;
                        avgY += majorPos.y;
                        count++;
                    }
                });
                
                if (count > 0) {
                    avgX /= count;
                    avgY /= count;
                    
                    // Ensure minimum distance from major nodes
                    let dx = minorPos.x - avgX;
                    let dy = minorPos.y - avgY;
                    const dist = Math.sqrt(dx*dx + dy*dy);
                    
                    // Find optimal position avoiding other majors
                    let bestPos = { x: (minorPos.x + avgX) / 2, y: (minorPos.y + avgY) / 2 };
                    let bestScore = -Infinity;
                    
                    // Test 8 possible positions around average
                    for (let angle = 0; angle < Math.PI * 2; angle += Math.PI / 4) {
                        const testPos = {
                            x: avgX + Math.cos(angle) * MIN_DISTANCE,
                            y: avgY + Math.sin(angle) * MIN_DISTANCE
                        };
                        
                        // Check for collisions with other majors
                        let collision = false;
                        for (let j = 0; j < loader.majorCoords.length; j++) {
                            if (j === node.index) continue; // Skip self
                            const otherMajor = loader.majorCoords[j];
                            if (otherMajor[2] !== z) continue;
                            
                            const otherPos = calculatePosition(otherMajor[0], otherMajor[1]);
                            const distToOther = Math.sqrt(
                                Math.pow(testPos.x - otherPos.x, 2) +
                                Math.pow(testPos.y - otherPos.y, 2)
                            );
                            
                            if (distToOther < AVOIDANCE_RADIUS) {
                                collision = true;
                                break;
                            }
                        }
                        
                        // Prefer positions without collisions
                        const score = collision ? -1 : 1 + Math.random() * 0.1; // Small random factor
                        if (score > bestScore) {
                            bestScore = score;
                            bestPos = testPos;
                        }
                    }
                    
                    minorPos = bestPos;
                } else if (!minorPos) {
                    // Fallback position
                    minorPos = {
                        x: pos.x + 40 + Math.random() * 20,
                        y: pos.y + 40 + Math.random() * 20
                    };
                }
                
                // Create minor node element
                const minorElement = document.createElement('div');
                minorElement.className = 'node minor-node';
                minorElement.style.left = `${minorPos.x - 8}px`;
                minorElement.style.top = `${minorPos.y - 8}px`;
                minorElement.dataset.index = `minor-${minorIndex}`; // Add type prefix
                minorElement.dataset.originalIndex = minorIndex; // Store original index
                
                // Add index label
                const label = document.createElement('div');
                label.className = 'node-label';
                label.textContent = minorIndex;
                minorElement.appendChild(label);
                
                // Add hover event
                minorElement.addEventListener('mouseover', () => {
                    const info = loader.getMinorNodeInfo(minorIndex);
                    let html = `<h4>Minor Node ${minorIndex}</h4>`;
                    html += `<p><strong>Connected Majors:</strong> ${info.adjMajors.join(', ')}</p>`;
                    html += `<p><strong>Rewards:</strong><br>`;
                    
                    Object.entries(info.desc).forEach(([stat, value]) => {
                        if (value !== 0) {
                            html += `${stat}: ${value > 0 ? '+' : ''}${value}<br>`;
                        }
                    });
                    html += `</p>`;
                    
                    infoPanel.innerHTML = html;
                });
                
                // Add hover handlers before appending
                const highlightAdjacent = () => {
                    // Highlight this node
                    minorElement.classList.add('highlight');
                    
                    // Highlight connected majors
                    minorInfo.adjMajors.forEach(majorIdx => {
                        const major = loader.majorCoords[majorIdx];
                        if (major[2] === z) {
                            const majorPos = calculatePosition(major[0], major[1]);
                            const majorElement = document.querySelector(`.major-node[data-index="major-${majorIdx}"]`);
                            if (majorElement) {
                                majorElement.classList.add('highlight');
                                
                                // Highlight connecting edges
                                document.querySelectorAll('.edge, .minor-edge').forEach(edge => {
                                    if (edge.style.transform.includes(`rotate(${Math.atan2(majorPos.y - minorPos.y, majorPos.x - minorPos.x)}rad)`)) {
                                        edge.classList.add('highlight');
                                    }
                                });
                            }
                        }
                    });
                };
                
                const removeHighlight = () => {
                    minorElement.classList.remove('highlight');
                    document.querySelectorAll('.highlight').forEach(el => {
                        el.classList.remove('highlight');
                    });
                };
                
                minorElement.addEventListener('mouseover', () => {
                    highlightAdjacent();
                    // Original hover handler code here...
                });
                
                minorElement.addEventListener('mouseout', removeHighlight);
                
                canvas.appendChild(minorElement);
                
                // Draw connections with slight random offsets
                minorInfo.adjMajors.forEach(majorIdx => {
                    const major = loader.majorCoords[majorIdx];
                    if (major[2] === z) {
                        const majorPos = calculatePosition(major[0], major[1]);
                        const line = document.createElement('div');
                        line.className = 'minor-edge';
                        line.style.left = `${minorPos.x}px`;
                        line.style.top = `${minorPos.y}px`;
                        line.style.width = `${Math.sqrt(Math.pow(majorPos.x - minorPos.x, 2) + Math.pow(majorPos.y - minorPos.y, 2))}px`;
                        
                        // Add random offset to prevent perfect overlap
                        const offset = (Math.random() - 0.5) * 4;
                        line.style.transform = `rotate(${Math.atan2(majorPos.y - minorPos.y, majorPos.x - minorPos.x)}rad) translateY(${offset}px)`;
                        line.style.zIndex = '5'; // Below major nodes (z-index 10)
                        
                        canvas.appendChild(line);
                    }
                });
                
                drawnMinors.add(minorIndex);
                minorPositions.set(minorIndex, minorPos);
            }
        });
    });
    
    // Minor nodes drawing is now implemented above
}

function initDataTables(loader) {
    // Init stats table
    const initStatTable = document.getElementById('init-stat-table');
    const initStatHeaders = ['Attribute', 'Value'];
    createTableHeader(initStatTable, initStatHeaders);
    
    Object.entries(loader.initStat).forEach(([key, value]) => {
        const row = initStatTable.insertRow();
        row.insertCell().textContent = key;
        row.insertCell().textContent = value;
    });
    
    // Level up table
    const levelupTable = document.getElementById('levelup-desc-table');
    const levelupHeaders = ['Level', 'Minor Reward', 'Exp Needed', 'Clear Exp'];
    createTableHeader(levelupTable, levelupHeaders);
    
    loader.levelupDesc.forEach((desc, index) => {
        const row = levelupTable.insertRow();
        row.insertCell().textContent = index + 1;
        const minorDesc = loader.minorDesc[desc.minor];
        const minorCell = row.insertCell();
        minorCell.innerHTML = `Minor ${desc.minor}<br>`;
        Object.entries(minorDesc).forEach(([stat, value]) => {
            if (value !== 0) {
                minorCell.innerHTML += `${stat}: ${value > 0 ? '+' : ''}${value}<br>`;
            }
        });
        row.insertCell().textContent = desc.need;
        row.insertCell().textContent = desc.clear ? 'Yes' : 'No';
    });
    
    // Enemy data table
    const enemyTable = document.getElementById('enemy-data-table');
    const enemyHeaders = ['ID', 'HP', 'ATK', 'DEF', 'AT Times', 'EXP', 'Special'];
    createTableHeader(enemyTable, enemyHeaders);
    
    loader.enemyData.forEach((enemy, index) => {
        const row = enemyTable.insertRow();
        row.insertCell().textContent = index;
        row.insertCell().textContent = enemy.hp;
        row.insertCell().textContent = enemy.atk;
        row.insertCell().textContent = enemy.def;
        row.insertCell().textContent = enemy.attimes;
        row.insertCell().textContent = enemy.exp;
        
        const specials = [];
        if (enemy.magic) specials.push('Magic');
        if (enemy.solid) specials.push('Solid');
        if (enemy.speedy) specials.push('Speedy');
        if (enemy.nobomb) specials.push('NoBomb');
        row.insertCell().textContent = specials.join(', ');
    });
}

function createTableHeader(table, headers) {
    const headerRow = table.insertRow();
    headers.forEach(header => {
        const th = document.createElement('th');
        th.textContent = header;
        headerRow.appendChild(th);
    });
}

function initTabs() {
    const tabs = document.querySelectorAll('.tab');
    tabs.forEach(tab => {
        tab.addEventListener('click', () => {
            // Remove active class from all tabs and contents
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
            
            // Add active class to clicked tab and corresponding content
            tab.classList.add('active');
            const target = document.getElementById(tab.dataset.target);
            if (target) target.classList.add('active');
        });
    });
}
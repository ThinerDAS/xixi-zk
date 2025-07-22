class MotadataLoader {
    constructor() {
        this.majorAdj = [];
        this.majorMinorAdj = [];
        this.majorDesc = [];
        this.minorDesc = [];
        this.enemyData = [];
        this.initStat = {};
        this.levelupDesc = [];
        this.majorCoords = [];
    }

    async loadFromJson(jsonPath) {
        try {
            const response = await fetch(jsonPath);
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            const data = await response.json();
            
            this.majorAdj = data.major_adj || [];
            this.majorMinorAdj = data.major_minor_adj || [];
            this.majorDesc = data.major_desc || [];
            this.minorDesc = data.minor_desc || [];
            this.enemyData = data.enemy_data || [];
            this.initStat = data.init_stat || {};
            this.levelupDesc = data.levelup_desc || [];
            this.majorCoords = data.major_coords || [];
            
            if (this.majorCoords.length === 0) {
                console.warn('No major_coords found in data. Did you set DEMO_MODE=1?');
            }
            
            return this;
        } catch (error) {
            console.error('Error loading JSON:', error);
            throw error;
        }
    }

    getLayers() {
        const layers = new Set();
        this.majorCoords.forEach(coord => layers.add(coord[2])); // z is at index 2
        return Array.from(layers).sort((a, b) => a - b);
    }

    getNodesByLayer(z) {
        return this.majorCoords
            .map((coord, index) => ({...coord, index}))
            .filter(coord => coord[2] === z);
    }

    getMajorNodeInfo(index) {
        return {
            coords: this.majorCoords[index],
            desc: this.majorDesc[index],
            adjMajors: this.majorAdj[index],
            adjMinors: this.majorMinorAdj[index]
        };
    }

    getMinorNodeInfo(index) {
        const adjMajors = [];
        this.majorMinorAdj.forEach((minors, majorIndex) => {
            if (minors.includes(index)) {
                adjMajors.push(majorIndex);
            }
        });
        
        return {
            desc: this.minorDesc[index],
            adjMajors: adjMajors
        };
    }
}

// Export for browser
if (typeof window !== 'undefined') {
    window.MotadataLoader = MotadataLoader;
}
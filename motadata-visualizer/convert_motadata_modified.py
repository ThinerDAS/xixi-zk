import json
import os
from pathlib import Path
from typing import List, Dict, Tuple, Union

DEMO_MODE = os.environ.get('DEMO_MODE') == '1'

# Data class for loading motadata.py
class Motadata:
    def __init__(self):
        # Major node adjacency list [[predecessor nodes]]
        self.major_adj: List[List[int]] = []
        # Major-minor node mapping [[minor_id]]
        self.major_minor_adj: List[List[int]] = []
        # Major node effects [("enemy",id) or ("delta",{attr:val})]
        self.major_desc: List[Tuple[str, Union[int, Dict]]] = []
        # Minor node rewards [{"atk":1, "def":2}]
        self.minor_desc: List[Dict[str, int]] = []
        # Enemy data
        self.enemy_data: List[Dict] = []
        # Initial state
        self.init_stat: Dict[str, int] = {}
        # Level up requirements
        self.levelup_desc: List[Dict] = []
        # Major node coordinates (only in demo mode)
        self.major_coords: List[Tuple[int, int, int]] = []

def load_motadata(motadata_path: str) -> Motadata:
    """Load game data from motadata.py"""
    import importlib.util
    import sys
    
    # Dynamically import motadata module
    spec = importlib.util.spec_from_file_location("motadata", motadata_path)
    motadata = importlib.util.module_from_spec(spec)
    sys.modules["motadata"] = motadata
    spec.loader.exec_module(motadata)
    
    data = Motadata()
    data.major_adj = motadata.major_adj
    data.major_minor_adj = motadata.major_minor_adj
    data.major_desc = motadata.major_desc
    data.minor_desc = motadata.minor_desc
    data.enemy_data = motadata.enemy_data

    # Load major_coords if in demo mode
    if DEMO_MODE:
        data.major_coords = motadata.major_coords

    # Medium difficulty adjustment
    #for i in data.enemy_data:
    #    if i['nobomb']:
    #        i['hp'] = 1
    # Make boss not decisive to allow smaller score
    #print(data.enemy_data[data.major_desc[1][1]])
    data.enemy_data[data.major_desc[1][1]]['hp'] = 1

    data.init_stat = motadata.init_stat
    data.levelup_desc = motadata.levelup_desc
    
    return data

def convert_to_rust_struct(data: Motadata) -> Dict:
    """Convert Python data structure to Rust-compatible format"""
    return {
        "major_adj": data.major_adj,
        "major_minor_adj": data.major_minor_adj,
        "major_desc": [
            {
                "Enemy": m[1]
            } if m[0] == "enemy" else {
                "Delta": [
                    (k, v)
                    for k, v in m[1].items()
                ]
            }
            for m in data.major_desc
        ],
        "minor_desc": [
            {"atk": m.get("atk", 0), "def": m.get("def", 0), 
             "hp": m.get("hp", 0), "mdef": m.get("mdef", 0)}
            for m in data.minor_desc
        ],
        "enemy_data": [
            {
                "atk": e["atk"], "def": e["def"], "hp": e["hp"],
                "attimes": e["attimes"], "exp": e["exp"],
                "magic": bool(e.get("magic", 0)),
                "solid": bool(e.get("solid", 0)),
                "speedy": bool(e.get("speedy", 0)),
                "nobomb": bool(e.get("nobomb", 0))
            }
            for e in data.enemy_data
        ],
        "init_stat": {
            "hp": data.init_stat["hp"],
            "atk": data.init_stat["atk"],
            "def": data.init_stat["def"],
            "mdef": data.init_stat["mdef"],
            "exp": data.init_stat["exp"],
            "lv": data.init_stat["lv"],
            "salt": data.init_stat.get("salt", 0),
            "big_salt": data.init_stat.get("big_salt", 0)
        },
        "levelup_desc": [
            {"minor": l["minor"], "need": l["need"], "clear": bool(l["clear"])}
            for l in data.levelup_desc
        ],
        # Only include major_coords in demo mode
        **({"major_coords": data.major_coords} if DEMO_MODE else {})
    }

def save_as_json(data: Dict, output_path: str):
    """Save as JSON file (ensures stable output)"""
    # Save full format (with indentation)
    with open(output_path, "w") as f:
        json.dump(data, f, indent=2, sort_keys=True)
    
    # Save compact format (for embedding)
    compact_path = output_path.replace(".json", ".compact.json")
    with open(compact_path, "w") as f:
        json.dump(data, f, separators=(',', ':'), sort_keys=True)

import argparse

def main():
    """Main function to convert motadata to Rust-compatible format"""
    parser = argparse.ArgumentParser(description='Convert motadata to Rust-compatible format')
    parser.add_argument('input', help='Path to motadata.py')
    parser.add_argument('--json-output', default='game_data.json',
                       help='Output JSON file path (for debugging)')
    args = parser.parse_args()

    # Load game data
    motadata = load_motadata(args.input)
    
    # Convert to Rust-compatible format
    rust_data = convert_to_rust_struct(motadata)
    
    # Validate all required fields
    required_fields = [
        'major_adj', 'major_minor_adj', 'minor_desc',
        'enemy_data', 'init_stat', 'levelup_desc'
    ]
    for field in required_fields:
        if not rust_data.get(field):
            raise ValueError(f"Missing required field: {field}")

    # Save as JSON for debugging
    save_as_json(rust_data, args.json_output)
    print(f"Successfully converted {args.input} to {args.json_output}")

if __name__ == "__main__":
    main()

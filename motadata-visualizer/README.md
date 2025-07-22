# motadata visualizer

Use `convert_motadata_modified.py` instead of original `convert_motadata.py` for annotated json. Put the annotated json at `game_data.json` (which I already did).

To reproduce:
```
DEMO_MODE=1 python3 convert_motadata_modified.py ../json-generator/bigdata/motadata.py --json-output game_data.json
```

And then in this directory

```
python -m http.server
```



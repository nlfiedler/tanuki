# Synthetic Data

## NASNET labels

Claude Code converted the raw `nasnet-labels.txt` into the JSON formatted `labels-map.json` and added the additional fields.

Some judgement calls made by Claude regarding the NASNET labels.

- **Fruits and vegetables** (936–957) I categorized as _food_ rather than _plant_. The visual reality of a photo of a strawberry or a bell pepper is almost always "food sitting on a counter," not a plant in the ground.
- **Fungi** (991–997) I split: _agaric_ through _bolete_ (the recognizable mushroom shapes) → _mushroom/plant_, while coral fungus → _fungus/plant_. The _mushroom_ at index 947 is food (since "mushroom" there is the supermarket variety).
- _bubble_ (971) is in _nature_ because ImageNet groups it with landscape — it's almost always a soap-bubble shot, not a manufactured object. Could equally be object.
- _Christmas stocking_ (496), _teddy bear_ (850), _piggy bank_ (719) went into _decoration_ rather than _object_ since they're more ornamental than functional.
- **Bottles of beverages** (440, 441, 737, 907) I labeled as the beverage itself (_beer_, _soda_, _wine_) under _food_, not as containers, since "wine bottle" in a photo means wine.
- _crane_ (134) is the bird and _crane_ (517) is the machinery — duplicate display labels but different categories. NASNet can return either; you may want different display labels (crane (bird) vs crane (machine)) to disambiguate, or leave them as-is and rely on category to differentiate.
- _mortar_ (666) I treated as the military weapon (per ImageNet's intent), not the mortar-and-pestle.
- _sorrel_ (339) is the horse breed, not the plant.

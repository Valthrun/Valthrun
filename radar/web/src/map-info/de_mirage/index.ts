import { LoadedMap } from "..";
import kImageRadar from "./radar.png";
import kImageBuyZones from "./overlay_buyzones.png";

export default {
    mapName: "de_mirage",
    displayName: "Mirage",

    metaInfo: {
        "resolution": 5.02,

        "offset": {
            "x": 3240,
            "y": 3410
        },

        "floors": []
    },

    overlayBuyzones: kImageBuyZones,
    overlayRadar: kImageRadar,
} satisfies LoadedMap;
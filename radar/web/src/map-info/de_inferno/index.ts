import { LoadedMap } from "..";
import kImageRadar from "./radar.png";
import kImageBuyZones from "./overlay_buyzones.png";

export default {
    mapName: "de_inferno",
    displayName: "Inferno",

    metaInfo: {
        "resolution": 4.91,

        "offset": {
            "x": 2090,
            "y": 1150
        },

        "floors": []
    },

    overlayBuyzones: kImageBuyZones,
    overlayRadar: kImageRadar,
} satisfies LoadedMap;
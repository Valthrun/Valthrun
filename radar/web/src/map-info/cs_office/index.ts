import { LoadedMap } from "..";
import kImageRadar from "./radar.png";

export default {
    mapName: "de_office",
    displayName: "Office",

    metaInfo: {
        "resolution": 4.26,

        "offset": {
            "x": 1900,
            "y": 2425
        },

        "floors": []
    },

    overlayBuyzones: "empty",
    overlayRadar: kImageRadar,
} satisfies LoadedMap;
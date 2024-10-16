import { LoadedMap } from "..";
import kImageBuyZones from "./overlay_buyzones.png";
import kImageRadar from "./radar.png";

export default {
    mapName: "de_nuke",
    displayName: "Nuke",

    metaInfo: {
        resolution: 6.98,

        offset: {
            x: 3290,
            y: 5990,
        },

        floors: [
            {
                offset: {
                    x: 0,
                    y: -46,
                },

                zRange: {
                    min: -780,
                    max: -480,
                },
            },
        ],
    },

    overlayBuyzones: kImageBuyZones,
    overlayRadar: kImageRadar,
} satisfies LoadedMap;

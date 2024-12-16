import { LoadedMap } from "..";
import OfficialDefault from "./map_style_cs2.png";
import OfficialLower from "./map_style_cs2_lower.png";

export default {
    mapName: "ar_baggage",
    displayName: "Baggage",

    pos_x: -1316, // upper left world coordinate
    pos_y: 1288,
    scale: 2.539062,

    verticalSections: [
        {
            name: "default",
            altitudeMax: 10000,
            altitudeMin: -5,
        },
        {
            name: "lower",
            altitudeMax: -5,
            altitudeMin: -10000,
        }
    ],

    mapStyles: [
        {
            name: "Official",
            map: {
                default: OfficialDefault,
                lower: OfficialLower
            }
        }
    ]
} satisfies LoadedMap;

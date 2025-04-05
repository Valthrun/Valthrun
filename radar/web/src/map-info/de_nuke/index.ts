import { LoadedMap } from "..";
import OfficialDefault from "./map_style_cs2.png";
import OfficialLower from "./radar_1_lower.png";

export default {
    mapName: "de_nuke",
    displayName: "Nuke",

    pos_x: -3453, // upper left world coordinate
    pos_y: 2887,
    scale: 7,


    verticalSections: [
        {
            name: "default",
            altitudeMax: 10000,
            altitudeMin: -495,
        },
        {
            name: "lower",
            altitudeMax: -495,
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

import { LoadedMap } from "..";
import OfficialDefault from "./map_style_cs2.png";
import OfficialLower from "./map_style_cs2_lower.png";

export default {
    mapName: "de_train",
    displayName: "Train",

    pos_x: -2308, // upper left world coordinate
    pos_y: 2078,
    scale: 4.082077,



    verticalSections: [
        {
            name: "default",
            altitudeMax: 20000,
            altitudeMin: -50,
        },
        {
            name: "lower",
            altitudeMax: -50,
            altitudeMin: -5000,
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

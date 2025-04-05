import { LoadedMap } from "..";
import OfficialDefault from "./map_style_cs2.png";
import OfficialLower from "./map_style_cs2_lower.png";

export default {
    mapName: "de_vertigo",
    displayName: "Vertigo",

    pos_x: -3168, // upper left world coordinate
    pos_y: 1762,
    scale: 4,

    verticalSections: [
        {
            name: "default",
            altitudeMax: 20000,
            altitudeMin: 11700,
        },
        {
            name: "lower",
            altitudeMax: 11700,
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

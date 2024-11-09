import { LoadedMap } from "..";
import SimpleRadarDefault from "./map_style_simple_radar.png";
import SimpleRadarLower from "./radar_0_lower.png";
import OfficialDefault from "./map_style_cs2.png";
import OfficialLower from "./radar_1_lower.png";

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
            name: "SimpleRadar",
            map: {
                default: SimpleRadarDefault,
                lower: SimpleRadarLower
            }
        },
        {
            name: "Official",
            map: {
                default: OfficialDefault,
                lower: OfficialLower
            }
        }
    ]
} satisfies LoadedMap;

import { LoadedMap } from "..";
import SimpleRadarDefault from "./radar_0_default.png";
import SimpleRadarLower from "./radar_0_lower.png";
import OfficialDefault from "./radar_1_default.png";
import OfficialLower from "./radar_1_lower.png";

export default {
    mapName: "de_vertigo",
    displayName: "Vertigo",

    pos_x: -3168, // upper left world coordinate
    pos_y: 1762,
    scale: 4,

    verticalSections: {
        default: // use the primary radar image
        {
            altitudeMax: 11680,
            altitudeMin: 11485,
        },
        lower: // i.e. radar_x_lower.png
        {
            altitudeMax: 11485,
            altitudeMin: -10000,
        }
    },

    mapImages: [
        {
            name: "SimpleRadar",
            images:{
                default: SimpleRadarDefault,
                lower: SimpleRadarLower
            }
        },
        {
            name: "Official",
            images:{
                default: OfficialDefault,
                lower: OfficialLower
            }
        }
    ]
} satisfies LoadedMap;

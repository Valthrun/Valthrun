import { LoadedMap } from "..";
import SimpleRadarDefault from "./radar_0_default.png";
import OfficialDefault from "./radar_1_default.png";

export default {
    mapName: "de_cache",
    displayName: "Cache",

    pos_x: -2020, // upper left world coordinate
    pos_y: 2390,
    scale: 5.54,

    verticalSections: {
        default: // use the primary radar image
        {
            altitudeMax: 10000,
            altitudeMin: -10000,
        },
    },

    mapImages: [
        {
            name: "SimpleRadar",
            images:{
                default: SimpleRadarDefault,
            }
        },
        {
            name: "Official",
            images:{
                default: OfficialDefault,
            }
        }
    ]
} satisfies LoadedMap;

import { LoadedMap } from "..";
import OfficialDefault from "./map_style_cs2.png";
import SimpleRadarDefault from "../de_ancient/map_style_simple_radar.png";

export default {
    mapName: "cs_office",
    displayName: "Office",

    pos_x: -1838, // upper left world coordinate
    pos_y: 1858,
    scale: 4.1,

    verticalSections: [
        {
            name: "default",
            altitudeMax: 10000,
            altitudeMin: -10000,
        }
    ],

    mapStyles: [
        {
            name: "SimpleRadar",
            map: {
                default: SimpleRadarDefault,
            }
        },
        {
            name: "Official",
            map: {
                default: OfficialDefault,
            }
        }
    ]
} satisfies LoadedMap;

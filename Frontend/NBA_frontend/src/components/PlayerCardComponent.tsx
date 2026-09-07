import { PlayerCardStats } from "../types/types";
import { Position } from "../types/enums"
 
type CardProps = {
    rank:number,
    ppg:number,
    rpg:number,
    apg:number,
    pos:Position
}

const PlayerCardComponent = (cardProps:CardProps) => {
  return (
    <div>
        <div className="icon"></div>
        <p className="rank">{cardProps.rank}</p>

        <p className="ppg">{cardProps.ppg}</p>
        <p className="rpg">{cardProps.rpg}</p>
        <p className="apg">{cardProps.apg}</p>

    </div>
  )
}

export default PlayerCardComponent
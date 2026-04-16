
use serde::{Deserialize, Serialize};
use crate::data::player;
use crate::data::player::{NameData, Player};
use crate::randoms::choices;

#[derive(Serialize,Deserialize)]
pub struct Round {
    round:i8,
    players:Vec<Player>

}

#[derive(Serialize,Deserialize)]
pub struct Draft {
    growth_bias:f32,
    rounds:Vec<Round>,
    picks_per_round:i8


}



impl Draft {

    pub fn new_class(growth:f32)-> Draft{



        Draft {growth_bias:growth,rounds : Vec::new(),picks_per_round : 1}
    }

    pub fn new_picks_class(growth:f32,picks:i8)-> Draft{

        Draft {growth_bias:growth,rounds : Vec::new(),picks_per_round : picks}
    }

    pub fn new_draft_with_bias(bias:f32,teams:i8,names:&NameData)-> Draft{
        let mut d = Draft::new_class(bias);

        for i in 1..7{
            d.create_round(i);
        }

        d.create_class(bias,teams as i32,names);
        d



    }

    pub fn create_round(&mut self,round:i8){
        self.rounds.push(Round{round,players: Vec::new()})

    }

    pub fn create_class(&mut self, bias:f32, teams:i32,names:&NameData){
        let mut index = 0;
        let rounds = self.rounds.len();
        for round in &mut self.rounds {
            index += 1;
            round.generate_players(
                (self.picks_per_round as i32 * teams),
                index,
                /* i32 */rounds as i32,
                bias,
                names
            );
        }
    }



}

impl Round {

    pub fn generate_players(&mut self, amount:i32, round:i8,rounds:i32, bias: f32,names:&NameData){
        let mut pos = round as i32;
        for i in 0..=amount {
            let pos = round as i32 + i;
            println!("POS {}, MAX: {}",pos + ((round - 1)as i32 * amount ) * 2,(rounds as i32 + amount) * 20i32);
            let prospect = player::random_prospect(
                choices::distanced_biased_falloff_random(
                    1,
                    pos + ((round - 1)as i32 * amount ) * 2,
                    (rounds as i32 + amount) * 20i32,
                    bias,
                    /* f32 */32.0,
                ),

                rand::random_ratio(1,8),names
            );

            self.players.push(prospect);
        }
    }



}
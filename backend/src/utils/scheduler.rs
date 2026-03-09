use std::sync::Arc;
use tokio::time::{interval, Duration};
use crate::service::competitions::CompetitionsService;

/// Background task scheduler for competition generation
pub struct CompetitionScheduler {
    competitions_service: Arc<dyn CompetitionsService>,
}

impl CompetitionScheduler {
    pub fn new(competitions_service: Arc<dyn CompetitionsService>) -> Self {
        Self {
            competitions_service,
        }
    }

    /// Start the scheduler that checks and generates competitions every 6 hours
    pub fn start(self) {
        tokio::spawn(async move {
            // Run check every 6 hours (21600 seconds) to maintain 10 competitions
            let mut timer = interval(Duration::from_secs(21600));

            // Initial generation on startup (after a short delay to let services initialize)
            tokio::time::sleep(Duration::from_secs(5)).await;
            self.check_and_generate().await;

            loop {
                timer.tick().await;
                self.check_and_generate().await;
            }
        });
    }

    async fn check_and_generate(&self) {
        println!("[Competition Scheduler] Running competition check (every 6 hours)...");
        
        match self.competitions_service.generate_competitions_if_needed().await {
            Ok(new_competitions) => {
                if new_competitions.is_empty() {
                    println!("[Competition Scheduler] No new competitions needed (already have 10+)");
                } else {
                    println!(
                        "[Competition Scheduler] Generated {} new competition(s)",
                        new_competitions.len()
                    );
                    for comp in new_competitions {
                        println!(
                            "[Competition Scheduler] - Competition {} (Type {}, Fish {}, {} {}-{} hours)",
                            comp.competition_id,
                            comp.competition_type,
                            comp.target_fish_id,
                            comp.reward_currency,
                            (comp.end_time - comp.start_time).num_hours(),
                            comp.prize_pool.len()
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("[Competition Scheduler] Error generating competitions: {:?}", e);
            }
        }
    }
}

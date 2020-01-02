mod launchd;

use crate::launchd::ServiceInfo;
use launchd::ServiceKiller;
use launchd::ServiceRepository;
use launchd::ServiceScraper;
use regex::Regex;
use rot13::rot13 as rotate;
use std::thread;
use std::time;

fn main() {
    let user_service_repo = ServiceRepository::new(false);
    let root_service_repo = ServiceRepository::new(true);

    fn control_loop(service_repo: &ServiceRepository) -> time::Duration {
        if service_repo.as_root {
            eprint!("[root] ");
        } else {
            eprint!("[user] ");
        }

        match control_loop_do(&service_repo) {
            ControlLoopActionTaken::NoAction => {
                eprintln!("nothing done");
                time::Duration::from_secs(60)
            }

            ControlLoopActionTaken::Killed(KilledAction { services }) => {
                eprintln!("killed {} services", services.len());

                for service in services {
                    eprintln!("  - {}", service.label);
                    eprintln!();
                }

                time::Duration::from_secs(5)
            }
        }
    }

    loop {
        let time = vec![
            control_loop(&user_service_repo),
            control_loop(&root_service_repo),
        ]
        .iter()
        .copied()
        .min()
        .unwrap();

        eprintln!("sleepin' {} seconds", time.as_secs());

        thread::sleep(time);
    }
}

enum ControlLoopActionTaken {
    NoAction,
    Killed(KilledAction),
}

struct KilledAction {
    services: Vec<ServiceInfo>,
}

fn control_loop_do(service_repo: &ServiceRepository) -> ControlLoopActionTaken {
    let running_services = service_repo.list_running().unwrap();

    let services_regex: Vec<Regex> = ["wnzs", "geraqzvpeb", "grnzivrjre"]
        .iter()
        .map(|r| Regex::new(&rotate(r)).unwrap())
        .collect();

    let matching_services = filter_services(&running_services, &services_regex, &[]);

    match matching_services {
        services if services.len() == 0 => ControlLoopActionTaken::NoAction,
        services => {
            services.iter().for_each(|service| {
                service_repo.kill_by_label(&service.label).unwrap();
            });
            ControlLoopActionTaken::Killed(KilledAction { services })
        }
    }
}

fn filter_services(
    service_list: &[ServiceInfo],
    mark_list: &[Regex],
    white_list: &[&str],
) -> Vec<ServiceInfo> {
    service_list
        .iter()
        .filter(|service| {
            let marked = mark_list
                .iter()
                .any(|regex| regex.is_match(service.label.as_str()));

            let whitelisted = white_list
                .iter()
                .any(|exact_name| exact_name == &service.label);

            marked && !whitelisted
        })
        .cloned()
        .collect()
}

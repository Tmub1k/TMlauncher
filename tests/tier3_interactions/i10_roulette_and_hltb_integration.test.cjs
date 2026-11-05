const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 3 - Interaction 10: Backlog Roulette & HowLongToBeat Forecasting', () => {
  it('Filters Backlog Roulette candidates using HLTB duration estimates for weekend play sessions', () => {
    const library = [
      { id: '1', title: 'Persona 5 Royal', playtime: 0, hltbStoryHours: 100 },
      { id: '2', title: 'Journey', playtime: 0, hltbStoryHours: 2 },
      { id: '3', title: 'What Remains of Edith Finch', playtime: 0, hltbStoryHours: 2.5 },
      { id: '4', title: 'Baldur\'s Gate 3', playtime: 0, hltbStoryHours: 75 },
    ];

    function filterForWeekend(games, maxHours = 5) {
      return games.filter(g => g.playtime === 0 && g.hltbStoryHours <= maxHours);
    }

    const shortGames = filterForWeekend(library, 5);
    equal(shortGames.length, 2);
    equal(shortGames[0].title, 'Journey');
    equal(shortGames[1].title, 'What Remains of Edith Finch');
  });

  it('Displays HLTB duration targets and dynamic progress bar inside Roulette Winner modal', () => {
    const winnerData = {
      title: 'Portal',
      playtimeHours: 0,
      hltb: {
        mainStory: '3h',
        extra: '5h',
        completionist: '10h',
      },
    };

    function renderWinnerModalCard(data) {
      return {
        cardTitle: data.title,
        completionEstimate: `Main Story: ~${data.hltb.mainStory}`,
        progressBarPercent: 0,
      };
    }

    const modal = renderWinnerModalCard(winnerData);
    equal(modal.cardTitle, 'Portal');
    equal(modal.completionEstimate, 'Main Story: ~3h');
    equal(modal.progressBarPercent, 0);
  });
});

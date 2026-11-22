// Site footer.
//
// mdBook has no footer setting, and theme/index.hbs is deliberately not
// overridden here -- forking the page template would turn every mdBook upgrade
// into a diff against upstream, for markup this small. So the element is
// appended client-side instead.
//
// It attaches to #mdbook-content rather than <main> so that it sits below the
// previous/next chapter links, and so it inherits the right-hand padding that
// custom.css reserves for the pagetoc panel above 1500px -- centring it over
// the text column rather than over the viewport.
(function bookFooter() {
    const COPYRIGHT = '© Chris Martinez';

    // No year, matching LICENSE.txt, which does not carry one either. A
    // hard-coded year in a static site is only correct until January.
    const LINKS = [
        {
            text: 'MIT',
            href: 'https://github.com/commonsensesoftware/asp-rs-versioning/blob/main/LICENSE',
        },
        {
            text: 'GitHub',
            href: 'https://github.com/commonsensesoftware/asp-rs-versioning',
        },
    ];

    document.addEventListener('DOMContentLoaded', function () {
        const content = document.querySelector('#mdbook-content');

        if (content === null) {
            return;
        }

        const footer = document.createElement('footer');

        footer.className = 'book-footer';
        footer.append(COPYRIGHT);

        for (const link of LINKS) {
            const anchor = document.createElement('a');

            anchor.href = link.href;
            anchor.textContent = link.text;
            footer.append(' · ', anchor);
        }

        content.append(footer);
    });
})();

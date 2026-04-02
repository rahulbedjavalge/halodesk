<script lang="ts">
  type Section = {
    title: string;
    intent: string;
    officialDocuments: string[];
    unofficialDocuments: string[];
    steps: string[];
    officialLinks: { label: string; url: string }[];
  };

  type CountryPlan = {
    intro: string;
    sections: Section[];
  };

  const countryPlans: Record<'india' | 'germany', CountryPlan> = {
    india: {
      intro:
        'India starter path: register your entity, secure your brand, and protect original work using official portals with practical prep documents.',
      sections: [
        {
          title: 'Company Setup (Private Limited / LLP)',
          intent: 'Create a legal business entity and complete compliance basics.',
          officialDocuments: [
            'PAN and Aadhaar of all directors/partners',
            'Address proof and identity proof of directors/partners',
            'Digital Signature Certificate (DSC)',
            'Director Identification Number (DIN) for directors',
            'Registered office proof (rent agreement/NOC/utility bill)',
            'MOA and AOA for company incorporation'
          ],
          unofficialDocuments: [
            'Founder agreement and equity split sheet',
            'Cap table draft',
            'One-page business model summary',
            'Brand name shortlist with backup options'
          ],
          steps: [
            'Reserve company name using RUN/SPICe+ on MCA portal.',
            'Prepare incorporation docs and digital signatures.',
            'File incorporation through SPICe+ and linked forms.',
            'Receive Certificate of Incorporation, PAN, TAN and open bank account.',
            'Set up GST, bookkeeping and annual filing reminders.'
          ],
          officialLinks: [
            { label: 'MCA Portal', url: 'https://www.mca.gov.in/' },
            { label: 'Startup India', url: 'https://www.startupindia.gov.in/' }
          ]
        },
        {
          title: 'Trademark Registration',
          intent: 'Protect your brand name/logo in relevant classes.',
          officialDocuments: [
            'Applicant identity proof and address proof',
            'Brand wordmark/logo in image format',
            'Signed TM-48 (if using attorney)',
            'User affidavit (if claiming prior use)',
            'Goods/services classification details'
          ],
          unofficialDocuments: [
            'Brand usage examples (website, social, packaging)',
            'Competitor mark screening notes',
            'Short rationale for chosen classes'
          ],
          steps: [
            'Check brand availability via IP India trademark search.',
            'Choose classes and submit application with required docs.',
            'Track examination report and reply within deadline if objected.',
            'Monitor journal publication and opposition window.',
            'Receive registration certificate and keep renewal calendar.'
          ],
          officialLinks: [
            { label: 'IP India', url: 'https://ipindia.gov.in/' }
          ]
        },
        {
          title: 'Copyright Registration',
          intent: 'Protect original software, design, content, and creative assets.',
          officialDocuments: [
            'Applicant details and declaration',
            'NOC from author if applicant differs',
            'Copies of original work/source samples',
            'Power of attorney (if filed through agent)'
          ],
          unofficialDocuments: [
            'Version history or creation timeline',
            'Asset ownership tracker (design/code/content)',
            'Contributor agreements archive'
          ],
          steps: [
            'Prepare work samples and ownership evidence.',
            'File application on copyright portal with correct category.',
            'Respond to queries/hearing notices, if any.',
            'Obtain registration extract and archive records safely.'
          ],
          officialLinks: [
            { label: 'Copyright Office India', url: 'https://copyright.gov.in/' }
          ]
        }
      ]
    },
    germany: {
      intro:
        'Germany starter path: complete legal registration, brand filing, and IP protection with practical founder-ready document preparation.',
      sections: [
        {
          title: 'Company Setup (UG/GmbH)',
          intent: 'Form your company, register trade, and prepare tax setup.',
          officialDocuments: [
            'Valid ID/passport of shareholders and managing directors',
            'Articles of association (Gesellschaftsvertrag)',
            'Notarized incorporation deed',
            'Proof of business address in Germany',
            'Commercial register filings (Handelsregister)',
            'Tax registration questionnaire (Fragebogen zur steuerlichen Erfassung)'
          ],
          unofficialDocuments: [
            'Founders agreement and vesting terms',
            'Shareholding plan and governance notes',
            'Business activity description (for Gewerbeanmeldung)',
            'Internal compliance checklist'
          ],
          steps: [
            'Draft and notarize articles of association.',
            'Open bank account and deposit share capital (as applicable).',
            'Register with Handelsregister through notary process.',
            'Complete trade office registration (Gewerbeanmeldung).',
            'Register with tax office and arrange bookkeeping/payroll setup.'
          ],
          officialLinks: [
            { label: 'BMWK Existenzgründung', url: 'https://www.existenzgruendungsportal.de/' },
            { label: 'Handelsregister', url: 'https://www.handelsregister.de/' }
          ]
        },
        {
          title: 'Trademark Registration',
          intent: 'Secure your company name/logo in Germany and optionally EU scope.',
          officialDocuments: [
            'Applicant legal details and address',
            'Wordmark or logo representation',
            'List of goods/services classes (Nice classification)',
            'Authorization if filed by representative'
          ],
          unofficialDocuments: [
            'Brand distinctiveness notes',
            'Pre-filing similarity search results',
            'Expansion plan for EU trademark strategy'
          ],
          steps: [
            'Run preliminary search on DPMA register.',
            'Choose classes and submit filing to DPMA (or EUIPO for EU-wide).',
            'Pay fees and track publication/opposition timelines.',
            'Store registration proof and renewal schedule.'
          ],
          officialLinks: [
            { label: 'DPMA', url: 'https://www.dpma.de/' },
            { label: 'EUIPO', url: 'https://euipo.europa.eu/' }
          ]
        },
        {
          title: 'Copyright Essentials',
          intent: 'Document and defend ownership of software and creative output.',
          officialDocuments: [
            'Authorship and creation records',
            'Contracts assigning IP from employees/contractors',
            'Evidence of first publication/use date'
          ],
          unofficialDocuments: [
            'Source snapshots and release logs',
            'Design file archives',
            'Contributors and licensing inventory'
          ],
          steps: [
            'Maintain dated evidence showing original creation.',
            'Ensure contracts clearly assign transferable IP rights.',
            'Use legal notices and preserve publication trail for enforcement.'
          ],
          officialLinks: [
            { label: 'DPMA - Copyright Basics', url: 'https://www.dpma.de/english/' }
          ]
        }
      ]
    }
  };

  type Message = { role: 'user' | 'bot'; text: string };

  let selectedCountry: 'india' | 'germany' = 'india';
  let chatInput = '';
  let messages: Message[] = [
    {
      role: 'bot',
      text: 'Hi founder 👋 Tell me what you need (company setup, trademark, or copyright), and I will guide you for your selected country.'
    }
  ];

  const normalize = (value: string) => value.toLowerCase().trim();

  function getBotReply(input: string): string {
    const prompt = normalize(input);
    const country = selectedCountry === 'india' ? 'India' : 'Germany';
    const plan = countryPlans[selectedCountry];

    if (!prompt) {
      return 'Please type your question so I can help.';
    }

    if (prompt.includes('company') || prompt.includes('register') || prompt.includes('incorpor')) {
      const section = plan.sections[0];
      return `${country} company setup:\n1) ${section.steps[0]}\n2) ${section.steps[1]}\n3) ${section.steps[2]}\nCore docs: ${section.officialDocuments.slice(0, 3).join(', ')}.`;
    }

    if (prompt.includes('trademark') || prompt.includes('brand') || prompt.includes('logo')) {
      const section = plan.sections[1];
      return `${country} trademark path:\n1) ${section.steps[0]}\n2) ${section.steps[1]}\n3) ${section.steps[2]}\nKey docs: ${section.officialDocuments.slice(0, 3).join(', ')}.`;
    }

    if (prompt.includes('copyright') || prompt.includes('content') || prompt.includes('software')) {
      const section = plan.sections[2];
      return `${country} copyright path:\n1) ${section.steps[0]}\n2) ${section.steps[1]}\n3) ${section.steps[2]}\nKey docs: ${section.officialDocuments.slice(0, 2).join(', ')}.`;
    }

    if (prompt.includes('hello') || prompt.includes('hi') || prompt.includes('start')) {
      return `Great, let's start with ${country}. Choose a track: company setup, trademark, or copyright. I can also give you a document checklist.`;
    }

    return `I can help with company setup, trademark, and copyright for ${country}. Ask naturally, for example: "How do I start trademark filing?"`;
  }

  function sendMessage() {
    const text = chatInput.trim();
    if (!text) {
      return;
    }

    messages = [...messages, { role: 'user', text }];
    messages = [...messages, { role: 'bot', text: getBotReply(text) }];
    chatInput = '';
  }
</script>

<svelte:head>
  <title>Founder Launch Guide | India & Germany</title>
</svelte:head>

<main class="page">
  <section class="hero card">
    <p class="eyebrow">Founder Docs Assistant</p>
    <h1>Start your company paperwork with clarity</h1>
    <p>
      One practical path for each country. Begin with <strong>India</strong> or <strong>Germany</strong>, then follow
      clear steps for company setup, trademark, and copyright.
    </p>

    <div class="country-switch" role="group" aria-label="Select country">
      <button
        type="button"
        class:selected={selectedCountry === 'india'}
        on:click={() => (selectedCountry = 'india')}
      >
        India
      </button>
      <button
        type="button"
        class:selected={selectedCountry === 'germany'}
        on:click={() => (selectedCountry = 'germany')}
      >
        Germany
      </button>
    </div>

    <p class="country-intro">{countryPlans[selectedCountry].intro}</p>
  </section>

  <section class="guides">
    {#each countryPlans[selectedCountry].sections as section}
      <article class="card guide-card">
        <h2>{section.title}</h2>
        <p class="intent">{section.intent}</p>

        <div class="grid">
          <div>
            <h3>Official documents</h3>
            <ul>
              {#each section.officialDocuments as doc}
                <li>{doc}</li>
              {/each}
            </ul>
          </div>
          <div>
            <h3>Unofficial prep docs</h3>
            <ul>
              {#each section.unofficialDocuments as doc}
                <li>{doc}</li>
              {/each}
            </ul>
          </div>
        </div>

        <h3>Step-by-step</h3>
        <ol>
          {#each section.steps as step}
            <li>{step}</li>
          {/each}
        </ol>

        <div class="links">
          {#each section.officialLinks as link}
            <a href={link.url} target="_blank" rel="noreferrer">{link.label}</a>
          {/each}
        </div>
      </article>
    {/each}
  </section>

  <section class="card chatbot">
    <h2>Natural-language chatbot</h2>
    <p>Ask in plain language and get fast guidance for the selected country.</p>

    <div class="messages" aria-live="polite">
      {#each messages as message}
        <div class="bubble {message.role}">{message.text}</div>
      {/each}
    </div>

    <form
      class="chat-input"
      on:submit|preventDefault={() => {
        sendMessage();
      }}
    >
      <input
        type="text"
        bind:value={chatInput}
        placeholder="Example: How do I register a trademark?"
        aria-label="Chat input"
      />
      <button type="submit">Send</button>
    </form>
  </section>
</main>

<style>
  :global(body) {
    font-family: Inter, 'Segoe UI', system-ui, sans-serif;
    background: radial-gradient(circle at 20% 10%, #111827, #030712 55%);
    color: #f9fafb;
  }

  .page {
    max-width: 1100px;
    margin: 0 auto;
    padding: 2rem 1.25rem 3rem;
    display: grid;
    gap: 1.25rem;
  }

  .card {
    background: rgba(17, 24, 39, 0.88);
    border: 1px solid rgba(148, 163, 184, 0.2);
    border-radius: 1rem;
    padding: 1.25rem;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.25);
  }

  .eyebrow {
    color: #a5b4fc;
    text-transform: uppercase;
    letter-spacing: 0.09em;
    font-size: 0.75rem;
    margin-bottom: 0.25rem;
  }

  h1 {
    margin: 0.35rem 0 0.75rem;
    font-size: clamp(1.5rem, 4.5vw, 2.4rem);
    line-height: 1.2;
  }

  h2 {
    margin: 0;
    font-size: 1.25rem;
  }

  .country-switch {
    display: inline-flex;
    background: rgba(2, 6, 23, 0.7);
    border: 1px solid rgba(148, 163, 184, 0.2);
    border-radius: 999px;
    padding: 0.2rem;
    gap: 0.35rem;
    margin-top: 0.8rem;
  }

  .country-switch button,
  .chat-input button {
    cursor: pointer;
    border: none;
    border-radius: 999px;
    padding: 0.55rem 0.9rem;
    color: #e5e7eb;
    background: transparent;
    font-weight: 600;
  }

  .country-switch button.selected,
  .chat-input button {
    background: linear-gradient(120deg, #4f46e5, #7c3aed);
  }

  .country-intro {
    margin-top: 0.9rem;
    color: #cbd5e1;
  }

  .guides {
    display: grid;
    gap: 1rem;
  }

  .intent {
    color: #cbd5e1;
    margin: 0.4rem 0 0.9rem;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 1rem;
  }

  h3 {
    margin: 1rem 0 0.55rem;
    font-size: 0.98rem;
    color: #c7d2fe;
  }

  ul,
  ol {
    margin: 0;
    padding-left: 1rem;
    color: #e5e7eb;
    display: grid;
    gap: 0.4rem;
  }

  .links {
    display: flex;
    flex-wrap: wrap;
    gap: 0.6rem;
    margin-top: 1rem;
  }

  .links a {
    color: #93c5fd;
    text-decoration: none;
    border: 1px solid rgba(147, 197, 253, 0.45);
    padding: 0.35rem 0.65rem;
    border-radius: 999px;
    font-size: 0.85rem;
  }

  .chatbot p {
    margin-top: 0.5rem;
    color: #cbd5e1;
  }

  .messages {
    margin-top: 0.95rem;
    min-height: 160px;
    max-height: 300px;
    overflow: auto;
    border-radius: 0.9rem;
    border: 1px solid rgba(148, 163, 184, 0.25);
    background: rgba(2, 6, 23, 0.65);
    padding: 0.85rem;
    display: grid;
    gap: 0.6rem;
  }

  .bubble {
    max-width: 90%;
    white-space: pre-wrap;
    padding: 0.65rem 0.8rem;
    border-radius: 0.75rem;
    font-size: 0.92rem;
    line-height: 1.35;
  }

  .bubble.user {
    justify-self: end;
    background: rgba(79, 70, 229, 0.85);
  }

  .bubble.bot {
    justify-self: start;
    background: rgba(30, 41, 59, 0.95);
    border: 1px solid rgba(148, 163, 184, 0.2);
  }

  .chat-input {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 0.6rem;
    margin-top: 0.85rem;
  }

  .chat-input input {
    border: 1px solid rgba(148, 163, 184, 0.25);
    background: rgba(2, 6, 23, 0.8);
    border-radius: 0.75rem;
    color: #f8fafc;
    padding: 0.72rem 0.8rem;
    font: inherit;
  }

  .chat-input input:focus-visible,
  .country-switch button:focus-visible,
  .chat-input button:focus-visible,
  .links a:focus-visible {
    outline: 2px solid #a5b4fc;
    outline-offset: 2px;
  }

  @media (max-width: 640px) {
    .page {
      padding: 1rem 0.8rem 2rem;
    }
  }
</style>

// Aeonmi Monaco language definition (initial stub)
// Registers 'aeonmi' language with basic tokenization.
(function(){
  if (typeof monaco === 'undefined') { return; }
  monaco.languages.register({ id: 'aeonmi' });
  monaco.languages.setMonarchTokensProvider('aeonmi', {
    defaultToken: '',
    tokenPostfix: '.ai',
    keywords: [
      'let','fn','function','return','if','else','while','for','log','native','quantum','true','false'
    ],
    operators: ['=','==','!=','+','-','*','/','%','&&','||','!','<','<=','>','>='],
    symbols: /[=><!~?:&|+\-*\/\^%]+/,
    tokenizer: {
      root: [
        [/\/[/*].*/, 'comment'],
        [/\b\d+(?:\.\d+)?\b/, 'number'],
        [/"([^"\\]|\\.)*"/, 'string'],
        [/'([^'\\]|\\.)*'/, 'string'],
        [/`([^`\\]|\\.)*`/, 'string'],
        [/\b(true|false)\b/, 'boolean'],
        [/(superpose|entangle|measure|dod)/, 'keyword'],
        [/\b(function|fn|let|return|if|else|while|for|log|native|quantum)\b/, 'keyword'],
        [/\b[A-Za-z_][A-Za-z0-9_]*\b/, 'identifier'],
        [/\s+/, ''],
        [/@symbols/, { cases: { '@operators': 'operator', '@default': '' } }],
      ],
    }
  });
  monaco.languages.setLanguageConfiguration('aeonmi', {
    comments: { lineComment: '//', blockComment: ['/*','*/'] },
    brackets: [ ['{','}'], ['[',']'], ['(',')'] ],
    autoClosingPairs: [
      { open: '{', close: '}' },
      { open: '[', close: ']' },
      { open: '(', close: ')' },
      { open: '"', close: '"', notIn: ['string'] },
      { open: "'", close: "'", notIn: ['string'] },
      { open: '`', close: '`', notIn: ['string'] },
    ],
    surroundingPairs: [ ['{','}'], ['[',']'], ['(',')'], ['"','"'], ["'","'"], ['`','`'] ],
    indentationRules: {
      increaseIndentPattern: /{[^}"']*$/,
      decreaseIndentPattern: /^\s*}/,
    }
  });
})();

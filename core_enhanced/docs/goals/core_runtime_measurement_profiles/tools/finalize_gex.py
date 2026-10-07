"""Publish GEX10 self-verification; never publish an independent verdict."""
from evidence_common import ROOT, HERE, S, C, read, write, ref, selected

selection = selected()
coverage = read(S/'COVERAGE.json')
assert coverage['source_sha256'] == selection['source_sha256']
assert len(coverage['acceptances']) == len(coverage['validations']) == len(coverage['required_dimensions']) == 10
impact = read(S/'PREDECESSOR_IMPACT.json')
assert len(impact['conditions']) == 181 and len(impact['r06_acceptances']) == 31
handoff = read(HERE/'P2G_60_HANDOFF_001.json')['artifacts'][0]
write(S/'GEX10_RESULT.json',{'goal':'GOAL-CMRP-01','goal_set':'GS-CMRP-001','phase':'PHASE-CMRP-01',
      'mode':'goal-run','source_sha256':selection['source_sha256'],'handoff_ref':handoff['ref'],
      'state':'Implementation and self-verification complete; independent GEX20 pending',
      'goal_status':'Not independently Complete','independent_audit':False,
      'execution_nodes':[{ 'id':'WP-CMRP-'+name, 'status':'implemented and current self-verified'} for name in ['PROFILE','ACCOUNTING','COLLECTOR','PLAYBACK','VERIFY']],
      'selection':ref(S/'EVIDENCE_SELECTION.json'),'coverage':ref(S/'COVERAGE.json'),
      'predecessor_impact':ref(S/'PREDECESSOR_IMPACT.json'),'compatibility_adaptation':ref(C/'ADAPTATION_FINAL.json'),
      'canonical_documents':coverage['canonical_documentation'],
      'next':'Freeze exact target; nonparticipant GPT-6 Astra/High audits whole goal. Sol corrects findings only after pass closure. No U08 before Complete/High/open0.'})
print('PASSGEX10 self-verification; independent audit pending')

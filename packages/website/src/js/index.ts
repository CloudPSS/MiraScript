import PkgJson from '@site/package.json';
import ExecutionEnvironment from '@docusaurus/ExecutionEnvironment';

if (ExecutionEnvironment.canUseDOM) {
    // eslint-disable-next-line no-console
    console.log(
        `%cMiraScript Docs%c v${PkgJson.version}`,
        'color: #fff; font-weight: bold; background: linear-gradient(135deg, #6366f1, #8b5cf6); padding: 2px 4px; border-radius: 2px;',
        'color: inherit; background: none; ',
    );
}
